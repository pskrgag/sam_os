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

    pub fn populate_indential(&mut self, virt: &MemRange<VirtAddr>) {
        /* Lvl1 addresses 1 GB */
        for i in (0..virt.size() / (1 << 30)) {
            self.lvl1[l1_linear_offset(VirtAddr::from(usize::from(virt.start()) + (1 << 30) * i))] = PageBlock::new()
                .valid()
                .out_addr(PhysAddr::from((&self.lvl2 as *const _) as usize))
                .write();
        }

        dbg!("{} {} {} ", virt.start(), l1_linear_offset(virt.start()), l2_linear_offset(virt.start()));
       
        /* Lvl2 addresses 2 MB */
        for i in (0..virt.size() / (2 << 20)) {
            self.lvl2[l2_linear_offset(VirtAddr::from(usize::from(virt.start()) + (2 << 20) * i))] = PageBlock::new()
                .valid()
                .out_addr(usize::from(virt.start()).into())
                .write();
        }
    }
}
