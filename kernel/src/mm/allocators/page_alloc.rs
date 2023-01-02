// FIXME one day...
#[path = "../../arch/aarch64/qemu/config.rs"]
mod config;

use crate::{
    arch, arch::cpuid::Cpu, arch::PAGE_SIZE, kernel, lib::collections::vector::Vector, mm::types::*,
};
use bitmaps::Bitmap;
use lock_free_buddy_allocator::buddy_alloc::BuddyAlloc;
use spin::once::Once;

pub struct PageAlloc {
    pool: Vector<Bitmap<64>>,
    start: Pfn,
}

unsafe impl Send for PageAlloc {}

pub static PAGE_ALLOC: Once<BuddyAlloc<{ arch::PAGE_SIZE }, Cpu, alloc::alloc::Global>> =
    spin::Once::new();

pub fn init() {
    let alloc_start = PhysAddr::from(arch::ram_base() as usize + kernel::misc::image_size());
    println!(
        "{} {}",
        arch::ram_size() as usize,
        kernel::misc::image_size()
    );
    let alloc_size = arch::ram_size() as usize - kernel::misc::image_size();

    println!(
        "Page allocator start {:x} size {:x}",
        alloc_start.get(),
        alloc_size
    );

    PAGE_ALLOC.call_once(|| {
        BuddyAlloc::<PAGE_SIZE, Cpu, alloc::alloc::Global>::new(
            alloc_start.into(),
            alloc_size / arch::PAGE_SIZE,
            &alloc::alloc::Global,
        )
        .unwrap()
    });
}

pub fn page_allocator(
) -> &'static BuddyAlloc<'static, { arch::PAGE_SIZE }, Cpu, alloc::alloc::Global> {
    &PAGE_ALLOC.get().unwrap()
}
