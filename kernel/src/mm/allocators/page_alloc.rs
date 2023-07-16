use crate::{arch, arch::cpuid::CpuLayout, arch::PAGE_SIZE, kernel, mm::types::*};
use bitmaps::Bitmap;
use lock_free_buddy_allocator::buddy_alloc::BuddyAlloc;
use spin::once::Once;

use alloc::vec::Vec;

pub struct PageAlloc {
    pool: Vec<Bitmap<64>>,
    start: Pfn,
}

unsafe impl Send for PageAlloc {}

pub static PAGE_ALLOC: Once<BuddyAlloc<{ arch::PAGE_SIZE }, CpuLayout, alloc::alloc::Global>> =
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
            alloc_size / arch::PAGE_SIZE,
            &alloc::alloc::Global,
        )
        .unwrap()
    });
}

pub fn page_allocator(
) -> &'static BuddyAlloc<'static, { arch::PAGE_SIZE }, CpuLayout, alloc::alloc::Global> {
    &PAGE_ALLOC.get().unwrap()
}
