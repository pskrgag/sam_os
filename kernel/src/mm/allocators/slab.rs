use crate::{
    arch::PAGE_SIZE,
    kernel::locking::spinlock::Spinlock,
    mm::{
        allocators::page_alloc::page_allocator, paging::kernel_page_table::kernel_page_table,
        paging::page_table::MappingType, types::*, MemRange,
    },
};
use crate::kernel::misc::num_pages;

use core::{
    alloc::Layout,
    marker::PhantomData,
};

const MIN_SLAB_SIZE: usize = 8;

pub trait SlabPolicy {
    const MAX_SLABS: Option<usize> = None;
    type ObjectType;
}

pub struct DefaultPolicy {
    _unused: (),
}

impl SlabPolicy for DefaultPolicy {
    type ObjectType = u8;
}

pub struct SlabAllocator<P: SlabPolicy = DefaultPolicy> {
    slab_size: usize,
    freelist: FreeList,
    _p: PhantomData<P>,
}

struct FreeList {
    next: Option<&'static mut Slab>,
    base: VirtAddr,
}

struct Slab {
    next: Option<&'static mut Slab>,
}

static KERNEL_SLABS: [Spinlock<SlabAllocator>; 10] = [
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
];

impl<P: SlabPolicy> SlabAllocator<P> {
    pub const fn default() -> Self {
        Self {
            slab_size: 0,
            freelist: FreeList::default(),
            _p: PhantomData,
        }
    }

    pub fn owns(&self, ptr: *const u8) -> bool {
        if let Some(max) = P::MAX_SLABS {
            let va = self.freelist.base.to_raw::<u8>() as usize;
            let full = va + num_pages(max * self.slab_size) * PAGE_SIZE;

            va >= ptr as usize && (ptr as usize) < full
        } else {
            false
        }
    }

    pub fn new(size: usize) -> Option<Self> {
        Some(Self {
            slab_size: size,
            freelist: FreeList::new(size, P::MAX_SLABS)?,
            _p: PhantomData,
        })
    }

    pub fn alloc(&mut self) -> Option<*mut P::ObjectType> {
        match self
            .freelist
            .alloc()
            .map(|ptr| ptr as *mut Slab as *mut u8)
        {
            Some(ptr) => Some(ptr as *mut u8 as *mut P::ObjectType),
            None => {
                if P::MAX_SLABS.is_none() {
                    let new_list = FreeList::new(self.slab_size, None)?;
                    self.freelist.add_to_freelist(new_list.next.unwrap());

                    self.freelist
                        .alloc()
                        .map(|ptr: &mut Slab| ptr as *mut Slab as *mut u8 as *mut P::ObjectType)
                } else {
                    None
                }
            }
        }
    }

    pub fn free(&mut self, addr: *mut P::ObjectType) {
        self.freelist
            .add_to_freelist(unsafe { &mut *(addr as *mut Slab) });
    }
}

impl FreeList {
    /* Allocate one page for the beggining */
    pub fn new(size: usize, max_slabs: Option<usize>) -> Option<Self> {
        let pages = if let Some(m) = max_slabs {

            let full = m * size;
            num_pages(full)
        } else {
            1_usize
        };

        let pa: PhysAddr = page_allocator().alloc(pages)?.into();
        let mut list = Self::default();
        let mut va = VirtAddr::from(pa);
        let block_count = PAGE_SIZE / size;

        list.base = va;

        for _ in 0..block_count {
            let new = va.to_raw_mut::<Slab>();
            list.add_to_freelist(unsafe { &mut *new });
            va.add(size);
        }

        Some(list)
    }

    pub fn add_to_freelist(&mut self, new: &'static mut Slab) {
        match self.next.take() {
            Some(l) => {
                new.next = Some(l);
                self.next = Some(new);
            }
            None => self.next = Some(new),
        }
    }

    pub fn alloc(&mut self) -> Option<&mut Slab> {
        let next = self.next.take()?;

        self.next = next.next.take();
        Some(next)
    }

    pub const fn default() -> Self {
        Self { next: None, base: VirtAddr::new(0) }
    }
}

pub fn alloc(mut size: usize) -> Option<*mut u8> {
    size = core::cmp::max(size, MIN_SLAB_SIZE);

    let slab_index = (size.next_power_of_two().ilog2() as usize) - 3;

    if slab_index >= KERNEL_SLABS.len() {
        println!(
            "Too big allocation ({}) for kernel slabs! Please, add direct page alloc fallback",
            size
        );
        return None;
    }

    KERNEL_SLABS[slab_index].lock().alloc()
}

pub fn free(ptr: *mut u8, l: Layout) {
    let size = core::cmp::max(l.size(), MIN_SLAB_SIZE);

    let slab_index = (size.next_power_of_two().ilog2() as usize) - 3;
    if slab_index >= KERNEL_SLABS.len() {
        panic!();
    }

    KERNEL_SLABS[slab_index].lock().free(ptr);
}

pub fn init_kernel_slabs() -> Option<()> {
    let mut size = MIN_SLAB_SIZE;

    for i in &KERNEL_SLABS {
        (*i.lock()) = SlabAllocator::new(size)?;
        size = (size + 1).next_power_of_two();
    }

    crate::mm::allocators::allocator::BOOT_ALLOC_IS_DEAD
        .store(true, core::sync::atomic::Ordering::Relaxed);

    Some(())
}
