use crate::mm::allocators::page_alloc::page_allocator;
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::*;

pub struct StackLayout {
    base: VirtAddr,
    pages: usize,
}

impl StackLayout {
    pub fn new(num_pages: usize) -> Option<Self> {
        let stack = VirtAddr::from(PhysAddr::from(page_allocator().alloc(num_pages)?));

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
        page_allocator().free(self.base.into(), self.pages + 2);
    }
}
