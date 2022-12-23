use crate::{
    arch::PAGE_SIZE,
    mm::{
        allocators::page_alloc::PAGE_ALLOC,
        paging::{kernel_page_table::kernel_page_table, page_table::*},
        types::*,
    },
};

pub struct StackLayout {
    base: VirtAddr,
    pages: usize,
}

impl StackLayout {
    pub fn new(num_pages: usize) -> Option<Self> {
        let stack = VirtAddr::from(PAGE_ALLOC.lock().alloc_pages(num_pages + 2)?);

        kernel_page_table()
            .map(
                None,
                MemRange::new(VirtAddr::from(stack + PAGE_SIZE), num_pages * PAGE_SIZE),
                MappingType::KernelData,
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

        PAGE_ALLOC
            .lock()
            .free_pages(PhysAddr::from(self.base), self.pages + 2);
    }
}
