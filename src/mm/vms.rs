use crate::{
    mm::{
        types::*,
        allocators::page_alloc::PAGE_ALLOC,
    },
    lib::collections::list::List,
};

pub struct Vma {

}

pub struct Vms {
    start: VirtAddr,
    size: usize,
    table: PhysAddr,
    vmas: List<Vma>,
}

impl Vms {
    pub fn new(start: VirtAddr, size: usize, table: Option<PhysAddr>) -> Option<Self> {
        Some(Self {
            start: start,
            size: size,
            table: if let Some(t) = table { t } else { PAGE_ALLOC.lock().alloc_pages(1)? },
            vmas: List::new(),
        })
    }
}

impl Default for Vms {
    fn default() -> Self {
        Self::new(VirtAddr::new(0), 0, Some(PhysAddr::new(0))).unwrap()
    }
}
