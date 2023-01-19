use crate::{
    arch::{self, PAGE_SIZE},
    kernel::misc::num_pages,
    mm::{
        allocators::page_alloc::page_allocator,
        paging::kernel_page_table::kernel_page_table,
        paging::page_table::{MappingType, PageTable},
        types::*,
        vma_list::VmaList,
    },
};

pub struct Vms {
    ttbr0: Option<PageTable>,
}

impl Vms {
    pub fn empty() -> Option<Self> {
        Some(Self {
            ttbr0: page_allocator().alloc(1)?,
        })
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        if let Some(ttbr0) = &self.ttbr0 {
            Some(ttbr0.base())
        } else {
            None
        }
    }
}

impl Default for Vms {
    fn default() -> Self {
        Self::new(VirtAddr::new(0), 0, false).unwrap()
    }
}
