use crate::{arch, arch::cpuid::CpuLayout, kernel};
use bitmaps::Bitmap;
use lock_free_buddy_allocator::buddy_alloc::BuddyAlloc;
use spin::once::Once;

use alloc::vec::Vec;
use shared::vmm::types::*;
use shared::arch::PAGE_SIZE;

pub struct PageAlloc {
    pool: Vec<Bitmap<64>>,
    start: Pfn,
}

unsafe impl Send for PageAlloc {}

pub static PAGE_ALLOC: Once<BuddyAlloc<{ PAGE_SIZE }, CpuLayout, alloc::alloc::Global>> =
    spin::Once::new();

pub fn init() {
    let alloc_start = PhysAddr::from(arch::ram_base() as usize + kernel::misc::image_size());
    let alloc_size = arch::ram_size() as usize - kernel::misc::image_size();

    println!(
        "Page allocator start {:x} size {:x}",
        alloc_start.get(),
        alloc_size
    );

    PAGE_ALLOC.call_once(|| {
        BuddyAlloc::<PAGE_SIZE, CpuLayout, alloc::alloc::Global>::new(
            alloc_start.into(),
            alloc_size / PAGE_SIZE,
            &alloc::alloc::Global,
        )
        .unwrap()
    });
}

pub fn page_allocator(
) -> &'static BuddyAlloc<'static, { PAGE_SIZE }, CpuLayout, alloc::alloc::Global> {
    &PAGE_ALLOC.get().unwrap()
}
