use crate::{arch, arch::cpuid::CpuLayout, kernel::misc::{image_end_rounded, image_size}};
use lock_free_buddy_allocator::buddy_alloc::BuddyAlloc;
use spin::once::Once;

use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::*;

pub struct PageAllocWrapper<'a>(BuddyAlloc<'a, { PAGE_SIZE }, CpuLayout, alloc::alloc::Global>);

pub static PAGE_ALLOC: Once<PageAllocWrapper> = spin::Once::new();

pub fn init() {
    let alloc_start = PhysAddr::from(image_end_rounded());
    let alloc_size = arch::ram_size() as usize - image_size();

    println!(
        "Page allocator start {:x} size {:x}",
        alloc_start.get(),
        alloc_size
    );

    PAGE_ALLOC.call_once(|| {
        PageAllocWrapper(
            BuddyAlloc::<PAGE_SIZE, CpuLayout, alloc::alloc::Global>::new(
                alloc_start.into(),
                alloc_size / PAGE_SIZE,
                &alloc::alloc::Global,
            )
            .unwrap(),
        )
    });
}

impl PageAllocWrapper<'_> {
    pub fn alloc(&self, pages: usize) -> Option<PhysAddr> {
        let pa = self.0.alloc(pages)?;
        Some(pa.into())
    }

    pub fn free(&self, pa: PhysAddr, pages: usize) {
        self.0.free(pa.bits(), pages)
    }
}

pub fn page_allocator() -> &'static PageAllocWrapper<'static> {
    &PAGE_ALLOC.get().unwrap()
}
