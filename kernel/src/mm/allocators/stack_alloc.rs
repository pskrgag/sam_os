use crate::mm::{
    allocators::page_alloc::page_allocator, paging::kernel_page_table::kernel_page_table,
};
use shared::arch::PAGE_SIZE;
use shared::vmm::types::*;
use shared::vmm::MappingType;

pub struct StackLayout {
    base: VirtAddr,
    pages: usize,
}

impl StackLayout {
    pub fn new(num_pages: usize) -> Option<Self> {
        let stack = VirtAddr::from(PhysAddr::from(page_allocator().alloc(num_pages + 2)?));

        kernel_page_table()
            .map(
                None,
                MemRange::new(VirtAddr::from(stack + PAGE_SIZE), num_pages * PAGE_SIZE),
                MappingType::KERNEL_DATA,
            )
            .ok()?;

        Some(Self {
            base: stack,
            pages: num_pages,
        })
    }

    pub fn stack_head(&self) -> VirtAddr {
        VirtAddr::from(self.base + self.pages * PAGE_SIZE)
    }
}

impl Drop for StackLayout {
    fn drop(&mut self) {
        kernel_page_table()
            .unmap(MemRange::new(self.base, self.pages * PAGE_SIZE))
            .expect("Failed to unmap stack");

        page_allocator().free(self.base.into(), self.pages + 2);
    }
}
