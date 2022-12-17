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
    pub fn new(base: VirtAddr, pages: usize) -> Self {
        Self {
            base: base,
            pages: pages,
        }
    }

    pub fn stack_head(&self) -> VirtAddr {
        VirtAddr::from(self.base + self.pages * PAGE_SIZE)
    }
}

pub fn alloc_stack(num_pages: usize) -> Option<StackLayout> {
    let stack = VirtAddr::from(PAGE_ALLOC.lock().alloc_pages(num_pages + 2)?);

    kernel_page_table()
        .map(
            None,
            MemRange::new(VirtAddr::from(stack + PAGE_SIZE), num_pages * PAGE_SIZE),
            MappingType::KernelData,
        )
        .ok()?;

    Some(StackLayout::new(stack, num_pages))
}
