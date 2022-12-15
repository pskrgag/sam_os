use crate::{
    arch,
    lib::collections::list::List,
    mm::{
        allocators::page_alloc::PAGE_ALLOC,
        paging::{
            kernel_page_table::kernel_page_table,
            page_table::{MappingType, PageTable},
        },
        types::*,
    },
};

pub struct Vma {}

pub struct Vms {
    start: VirtAddr,
    size: usize,
    table: PhysAddr,
    vmas: List<Vma>,
}

impl Vms {
    pub fn new(start: VirtAddr, size: usize, user: bool) -> Option<Self> {
        Some(Self {
            start: start,
            size: size,
            table: if !user {
                kernel_page_table().base()
            } else {
                let pa = PAGE_ALLOC.lock().alloc_pages(1)?;
                kernel_page_table()
                    .map(
                        None,
                        MemRange::new(VirtAddr::from(pa), arch::PAGE_SIZE),
                        MappingType::KernelData,
                    )
                    .ok()?;

                pa
            },
            vmas: List::new(),
        })
    }
}

impl Default for Vms {
    fn default() -> Self {
        Self::new(VirtAddr::new(0), 0, false).unwrap()
    }
}
