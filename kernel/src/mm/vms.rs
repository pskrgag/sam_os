use crate::{
    arch::{self, PAGE_SIZE},
    mm::{
        paging::page_table::PageTable,
        types::*,
    },
};

pub struct Vms {
    ttbr0: PageTable,
}

impl Vms {
    pub fn empty() -> Option<Self> {
        Some(Self {
            ttbr0: PageTable::new(),
        })
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        panic!();
    }
}
