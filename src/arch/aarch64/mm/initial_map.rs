#[path = "../qemu/config.rs"]
mod config;

use crate::arch::mm::page_table::{
    PageBlock,
    PageTbl,
    l1_linear_offset,
    l2_linear_offset,
};
use core::mem;
use crate::mm::{
    types::{PhysAddr, VirtAddr, MemRange},
    page_table::PageTable,
};

#[repr(align(4096))]
#[repr(C)]
pub struct InitialPageTable {
    lvl1: [PageTbl; config::PT_LVL1_ENTIRES],
    lvl2: [PageBlock; config::PT_LVL2_ENTIRES],
}

#[used]
pub static mut initial_tt: InitialPageTable = InitialPageTable::default();

impl InitialPageTable {
    pub const fn default() -> Self {
         Self {
             lvl1: [PageTbl::new(); config::PT_LVL1_ENTIRES],
             lvl2: [PageBlock::new(); config::PT_LVL2_ENTIRES],
         }
    }

    pub fn populate_indential(&mut self, virt: &MemRange<VirtAddr>, device: bool) {
        /* Lvl1 addresses 1 GB */
        for i in (0..virt.size() / (1 << 30)) {
            self.lvl1[l1_linear_offset(VirtAddr::from(usize::from(virt.start()) + (1 << 30) * i))] = PageTbl::new()
                .valid()
                .next_lvl(PhysAddr::from((&self.lvl2 as *const _) as usize));
        }

        dbg!("{} {} {} ", virt.start(), l1_linear_offset(virt.start()), l2_linear_offset(virt.start()));
        dbg!("size {}", virt.size());

        /* Lvl2 addresses 2 MB */
        for i in (0..virt.size() / (2 << 20)) {
            let cur_addr = usize::from(virt.start()) + (2 << 20) * i;
            let idx = l2_linear_offset(VirtAddr::from(cur_addr));
            let tmp = PageBlock::new()
                .valid()
                .out_addr(cur_addr.into())
                .write();

            match device {
                true => tmp.device(),
                false => tmp.normal(),
            };
            
            self.lvl2[idx] = tmp;
            println!("entry[{}] = {:x}", idx, self.lvl2[idx].get());
        }
    }
}

impl PageTable for InitialPageTable {
    fn lvl1(&self) -> VirtAddr {
        VirtAddr::from_raw(&self.lvl1)
    }
    
    fn lvl2(&self) -> Option<VirtAddr> {
        Some(VirtAddr::from_raw(&self.lvl2))
    }

    fn entries_per_lvl(&self) -> usize {
        config::PT_LVL1_ENTIRES
    }
}
