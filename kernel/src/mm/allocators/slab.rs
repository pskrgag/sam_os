use crate::{
    arch::PAGE_SIZE,
    kernel::locking::spinlock::Spinlock,
    mm::{
        allocators::page_alloc::PAGE_ALLOC,
        paging::kernel_page_table::kernel_page_table,
        paging::page_table::{MappingType, PageTable},
        types::*,
        MemRange,
    },
};

use core::alloc::Layout;

const MIN_SLAB_SIZE: usize = 8;

pub struct SlabAllocator {
    slab_size: usize,
    freelist: FreeList,
}

struct FreeList {
    next: Option<&'static mut FreeList>,
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

impl SlabAllocator {
    pub const fn default() -> Self {
        Self {
            slab_size: 0,
            freelist: FreeList::default(),
        }
    }

    pub fn new(size: usize) -> Option<Self> {
        Some(Self {
            slab_size: size,
            freelist: FreeList::new(size)?,
        })
    }

    pub fn alloc(&mut self) -> Option<*mut u8> {
        match self
            .freelist
            .alloc()
            .map(|ptr| ptr as *mut FreeList as *mut u8)
        {
            Some(ptr) => Some(ptr),
            None => {
                let new_list = FreeList::new(self.slab_size)?;
                self.freelist.add_to_freelist(new_list.next.unwrap());

                self.freelist
                    .alloc()
                    .map(|ptr: &mut FreeList| ptr as *mut FreeList as *mut u8)
            }
        }
    }

    pub fn free(&mut self, addr: *mut u8) {
        self.freelist
            .add_to_freelist(unsafe { &mut *(addr as *mut FreeList) });
    }
}

impl FreeList {
    /* Allocate one page for the beggining */
    pub fn new(size: usize) -> Option<Self> {
        assert!(size.is_power_of_two());

        let pa = PAGE_ALLOC.lock().alloc_pages(1)?;
        let mut list = Self::default();
        let mut va = VirtAddr::from(pa);
        let block_count = PAGE_SIZE / size;

        kernel_page_table()
            .map(None, MemRange::new(va, PAGE_SIZE), MappingType::KernelData)
            .ok()?;

        for _ in 0..block_count {
            let new = va.to_raw_mut::<Self>();
            list.add_to_freelist(unsafe { &mut *new });
            va.add(size);
        }

        Some(list)
    }

    pub fn add_to_freelist(&mut self, new: &'static mut Self) {
        match self.next.take() {
            Some(l) => {
                new.next = Some(l);
                self.next = Some(new);
            }
            None => self.next = Some(new),
        }
    }

    pub fn alloc(&mut self) -> Option<&mut Self> {
        let next = self.next.take()?;

        self.next = next.next.take();
        Some(next)
    }

    pub const fn default() -> Self {
        Self { next: None }
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
        println!("Kernel slab {} initialized", size);
        size = (size + 1).next_power_of_two();
    }

    crate::mm::allocators::allocator::BOOT_ALLOC_IS_DEAD
        .store(true, core::sync::atomic::Ordering::Relaxed);

    Some(())
}
