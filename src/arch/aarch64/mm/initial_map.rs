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
    lvl2_normal: [PageBlock; config::PT_LVL2_ENTIRES],
    lvl2_device: [PageBlock; config::PT_LVL2_ENTIRES],
}

#[used]
pub static mut initial_tt: InitialPageTable = InitialPageTable::default();

impl InitialPageTable {
    pub const fn default() -> Self {
         Self {
             lvl1: [PageTbl::new(); config::PT_LVL1_ENTIRES],
             lvl2_normal: [PageBlock::new(); config::PT_LVL2_ENTIRES],
             lvl2_device: [PageBlock::new(); config::PT_LVL2_ENTIRES],
         }
    }

    pub fn populate_indential(&mut self, virt: &MemRange<VirtAddr>, device: bool) {
        let mut size_to_map = usize::from(virt.size());
        let mut curr_addr = usize::from(virt.start());
        let next_lvl = match device {
            true => &mut self.lvl2_device,
            false => &mut self.lvl2_normal,
        };

        println!("Mapping 0x{:x} with size 0x{:x}", curr_addr, size_to_map);

        /* Lvl1 addresses 1 GB */
        while {
            let idx = l1_linear_offset(VirtAddr::from(usize::from(curr_addr)));
            self.lvl1[idx] = PageTbl::new()
                .valid()
                .next_lvl(PhysAddr::from((next_lvl as *const _) as usize));
       
            //println!("[0x{:x}] lvl1 entry[{}] = 0x{:x}", VirtAddr::from_raw(&self.lvl1).get(), idx, self.lvl1[idx].get());

            curr_addr += (1 << 30);

            if size_to_map <= (1 << 30) {
                false
            } else {
                size_to_map -= (1 << 30);
                true
            }
        } { }
        
        let mut size_to_map = usize::from(virt.size());
        let mut curr_addr = usize::from(virt.start());

        /* Lvl2 addresses 2 MB */
        while  {
            let idx = l2_linear_offset(VirtAddr::from(curr_addr));
            let tmp = PageBlock::new()
                .valid()
                .out_addr(curr_addr.into())
                .write();

            match device {
                true => tmp.device(),
                false => tmp.normal(),
            };
            
            next_lvl[idx] = tmp;
            //println!("entry[{}] = 0x{:x}", idx, next_lvl[idx].get());

            curr_addr += (2 << 20);

            if size_to_map <= (2 << 20) {
                false
            } else {
                size_to_map -= (2 << 20);
                true
            }
        } { } 
    }
}

impl PageTable for InitialPageTable {
    fn lvl1(&self) -> VirtAddr {
        VirtAddr::from_raw(&self.lvl1)
    }
    
    fn lvl2(&self) -> Option<VirtAddr> {
        Some(VirtAddr::from_raw(&self.lvl2_normal))
    }

    fn entries_per_lvl(&self) -> usize {
        config::PT_LVL1_ENTIRES
    }
}
