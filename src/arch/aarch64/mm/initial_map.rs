#[path = "../qemu/config.rs"]
mod config;

use crate::arch::mm::page_table::{
    PageBlock,
    l1_linear_offset,
    l2_linear_offset,
};
use core::mem;
use crate::mm::types::{PhysAddr, VirtAddr, MemRange};

pub struct InitialPageTable {
    lvl1: [PageBlock; config::PT_LVL1_ENTIRES],
    lvl2: [PageBlock; config::PT_LVL2_ENTIRES],
}

#[used]
pub static mut initial_tt: InitialPageTable = InitialPageTable::default();

impl InitialPageTable {
    pub const fn default() -> Self {
         Self {
             lvl1: [PageBlock::new(); config::PT_LVL1_ENTIRES],
             lvl2: [PageBlock::new(); config::PT_LVL2_ENTIRES],
         }
    }

    pub fn populate(&mut self, phys: &MemRange<PhysAddr>, virt: &MemRange<VirtAddr>) {
         assert_eq!(phys.size(), virt.size());

         self.lvl1[l1_linear_offset(virt.start())] = PageBlock::new()
             .valid()
             .out_addr(PhysAddr::from((&self.lvl2 as *const _) as usize))
             .write();
         
        self.lvl2[l2_linear_offset(virt.start())] = PageBlock::new()
             .valid()
             .out_addr(phys.start())
             .write();
    }
}
