#[path = "../qemu/config.rs"]
mod config;

use crate::{
        arch::mm::page_table::{
        PageBlock,
        PageTbl,
        l1_linear_offset,
        l2_linear_offset,
    },
    mm::{
        types::{PhysAddr, VirtAddr, MemRange},
        page_table::PageTable,
    },
    arch::{
        PT_LVL1_ENTIRES,
        PT_LVL2_ENTIRES,
    },
    kernel::locking::fake_lock::FakeLock
};

/* Any idea how to use crate::arch::PAGE_SIZE? */
#[repr(align(4096))]
#[repr(C)]
pub struct InitialPageTable {
    lvl1: [PageTbl; PT_LVL1_ENTIRES],
    lvl2_normal: [PageBlock; PT_LVL2_ENTIRES],
    lvl2_device: [PageBlock; PT_LVL2_ENTIRES],
}

unsafe impl Sync for InitialPageTable { }
unsafe impl Send for InitialPageTable { }

pub static INITIAL_TT: FakeLock<InitialPageTable> = FakeLock::new(InitialPageTable::default());

impl InitialPageTable {
    pub const fn default() -> Self {
         Self {
             lvl1: [PageTbl::new(); PT_LVL1_ENTIRES],
             lvl2_normal: [PageBlock::new(); PT_LVL2_ENTIRES],
             lvl2_device: [PageBlock::new(); PT_LVL2_ENTIRES],
         }
    }

    pub fn populate_indential(&mut self, virt: &MemRange<VirtAddr>, device: bool) {
        let mut size_to_map = usize::from(virt.size());
        let mut curr_addr = usize::from(virt.start());
        let next_lvl = match device {
            true => &mut self.lvl2_device,
            false => &mut self.lvl2_normal,
        };

        println!("Mapping 0x{:x} -> 0x{:x} as {}", curr_addr, curr_addr, if device { "device" } else { "normal" });

        /* Lvl1 addresses 1 GB */
        while {
            let idx = l1_linear_offset(VirtAddr::from(usize::from(curr_addr)));
            self.lvl1[idx] = PageTbl::new()
                .valid()
                .next_lvl(PhysAddr::from((next_lvl as *const _) as usize));
       
            curr_addr += 1 << 30;

            if size_to_map <= 1 << 30 {
                false
            } else {
                size_to_map -= 1 << 30;
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

            curr_addr += 2 << 20;

            if size_to_map <= 2 << 20 {
                false
            } else {
                size_to_map -= 2 << 20;
                true
            }
        } { } 
    }
}

impl PageTable for InitialPageTable {
    fn lvl1(&self) -> VirtAddr {
        VirtAddr::from_raw(&self.lvl1)
    }

    fn entries_per_lvl(&self) -> usize {
        PT_LVL1_ENTIRES
    }
}
