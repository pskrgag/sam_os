use crate::{kernel::locking::spinlock::Spinlock, mm::allocators::page_alloc::page_allocator};

use core::alloc::Layout;
use rtl::vmm::alloc::BackendAllocator;
use rtl::vmm::slab::SlabAllocator;
use rtl::vmm::types::*;
use rtl::arch::PAGE_SIZE;

const MIN_SLAB_SIZE: usize = 8;

static KERNEL_SLABS: [Spinlock<SlabAllocator<PMMBackend>>; 10] = [
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

pub struct PMMBackend;
pub const PMMBackendImpl: PMMBackend = PMMBackend {};

unsafe impl Sync for PMMBackend {}

impl BackendAllocator for PMMBackend {
    fn allocate(&self, num_pages: usize) -> Option<*mut u8> {
        let pa = page_allocator().alloc(num_pages)?;
        let va = VirtAddr::from(pa);

        unsafe { va.as_slice_mut::<u8>(PAGE_SIZE).fill(0x00) };

        Some(va.to_raw_mut::<u8>())
    }

    fn free(&self, p: *const u8, num_pages: usize) {
        let pa: PhysAddr = PhysAddr::from(VirtAddr::from_raw(p));
        page_allocator().free(pa, num_pages);
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
        (*i.lock()) = SlabAllocator::new(size, &PMMBackendImpl)?;
        println!("Kernel slab {} initialized", size);
        size = (size + 1).next_power_of_two();
    }

    crate::mm::allocators::allocator::BOOT_ALLOC_IS_DEAD
        .store(true, core::sync::atomic::Ordering::Relaxed);

    Some(())
}
