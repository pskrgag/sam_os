use crate::kernel::tasks::task::kernel_task;
use crate::{kernel::locking::spinlock::Spinlock, mm::allocators::page_alloc::page_allocator};
use core::alloc::Layout;
use core::ptr::NonNull;
use hal::address::*;
use hal::arch::PAGE_SIZE;
use rtl::vmm::MappingType;

const MIN_SLAB_SIZE: usize = 8;

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

pub fn alloc(size: usize) -> Option<*mut u8> {
    let size = size.max(MIN_SLAB_SIZE);
    let slab_index = (size.next_power_of_two().ilog2() as usize) - 3;

    if slab_index >= KERNEL_SLABS.len() {
        return kernel_task()
            .vms()
            .vm_allocate(size, MappingType::Data)
            .map(|x| x.to_raw_mut())
            .ok();
    }

    KERNEL_SLABS[slab_index].lock().alloc()
}

pub fn free(ptr: *mut u8, l: Layout) {
    let size = l.size().max(MIN_SLAB_SIZE);

    let slab_index = (size.next_power_of_two().ilog2() as usize) - 3;
    if slab_index >= KERNEL_SLABS.len() {
        kernel_task().vms().vm_free(ptr.into(), l.size()).ok();
    } else {
        unsafe { KERNEL_SLABS[slab_index].lock().free(ptr) }
    }
}

pub fn init_kernel_slabs() -> Option<()> {
    let mut size = MIN_SLAB_SIZE;

    for i in &KERNEL_SLABS {
        (*i.lock()) = SlabAllocator::new(size)?;
        size = (size + 1).next_power_of_two();
    }

    Some(())
}

pub struct SlabAllocator {
    slab_size: usize,
    freelist: FreeList,
}

struct FreeList {
    next: Option<NonNull<FreeList>>,
}

unsafe impl Send for FreeList {}

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
        match self.freelist.alloc().map(|ptr| ptr as *mut u8) {
            Some(ptr) => Some(ptr),
            None => {
                let new_list = FreeList::new(self.slab_size)?;

                unsafe {
                    self.freelist.add_to_freelist(new_list.next.unwrap());

                    self.freelist
                        .alloc()
                        .map(|ptr: *mut FreeList| ptr as *mut u8)
                }
            }
        }
    }

    pub unsafe fn free(&mut self, addr: *mut u8) {
        unsafe {
            debug_assert!(!addr.is_null());
            debug_assert!((addr as *mut FreeList).is_aligned());

            let slice = core::slice::from_raw_parts_mut(addr, self.slab_size);
            slice.fill(0xa5);
            self.freelist
                .add_to_freelist(NonNull::new_unchecked(addr as *mut FreeList));
        }
    }
}

impl FreeList {
    /* Allocate one page for the beginning */
    pub fn new(size: usize) -> Option<Self> {
        assert!(size.is_power_of_two());

        let mut va = VirtAddr::from(LinearAddr::from(page_allocator().alloc(1)?));
        let block_count = PAGE_SIZE / size;
        let mut list = Self::default();

        for _ in 0..block_count {
            let new = va.to_raw_mut::<Self>();

            unsafe { list.add_to_freelist(NonNull::new_unchecked(new)) };
            va.add(size);
        }

        Some(list)
    }

    pub unsafe fn add_to_freelist(&mut self, mut new: NonNull<Self>) {
        unsafe {
            match self.next.take() {
                Some(l) => {
                    new.as_mut().next = Some(l);
                    self.next = Some(new);
                }
                None => {
                    new.as_mut().next = None;
                    self.next = Some(new);
                }
            }
        }
    }

    pub fn alloc(&mut self) -> Option<*mut Self> {
        unsafe {
            let mut next = self.next.take()?;

            // println!("{:p} {:p}", next, &self.next);
            self.next = next.as_mut().next.take();
            Some(next.as_ptr())
        }
    }

    pub const fn default() -> Self {
        Self { next: None }
    }
}
