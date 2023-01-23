use crate::{
    arch::{self, PAGE_SIZE},
    mm::{paging::page_table::PageTable, types::*},
};
use alloc::sync::Arc;
use object_lib::object;

#[derive(object)]
pub struct Vms {
    ttbr0: PageTable,
}

impl Vms {
    pub fn empty() -> Option<VmsRef> {
        Some(Self::construct(Self {
            ttbr0: PageTable::new(),
        }))
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        panic!();
    }
}
