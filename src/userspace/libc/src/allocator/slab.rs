use super::backend::{SyscallBackend, SyscallBackendImpl};
use rtl::locking::fake_lock::FakeLock;
use rtl::vmm::slab::SlabAllocator;

const MIN_SLAB_SIZE: usize = 8;

static SLABS: [FakeLock<SlabAllocator<SyscallBackend>>; 10] = [
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
    FakeLock::new(SlabAllocator::default()),
];

pub fn alloc(mut size: usize) -> Option<*mut u8> {
    size = core::cmp::max(size, MIN_SLAB_SIZE);

    let slab_index = (size.next_power_of_two().ilog2() as usize) - 3;

    if slab_index >= SLABS.len() {
        None
    } else {
        SLABS[slab_index].get().alloc()
    }
}

pub fn free(ptr: *mut u8, l: alloc::alloc::Layout) {
    let size = core::cmp::max(l.size(), MIN_SLAB_SIZE);

    let slab_index = (size.next_power_of_two().ilog2() as usize) - 3;
    if slab_index >= SLABS.len() {
        panic!();
    }

    SLABS[slab_index].get().free(ptr);
}

pub fn init() -> Option<()> {
    let mut size = MIN_SLAB_SIZE;

    for i in &SLABS {
        (*i.get()) = SlabAllocator::new(size, &SyscallBackendImpl)?;
        size = (size + 1).next_power_of_two();
    }

    Some(())
}
