#[path = "../qemu/config.rs"]
mod config;

use crate::{
    arch::mm::page_table::{l1_linear_offset, l2_linear_offset, PageBlock, PageTbl},
    arch::{PT_LVL1_ENTIRES, PT_LVL2_ENTIRES},
    kernel::locking::fake_lock::FakeLock,
    mm::{
        page_table::{
            MappingType, MmError, PageTable, TranslationTableBlock, TranslationTableTable,
        },
        types::{MemRange, PhysAddr, VirtAddr},
    },
};

/* Any idea how to use crate::arch::PAGE_SIZE? */
#[repr(align(4096))]
#[repr(C)]
pub struct InitialPageTable {
    lvl1: [PageTbl; PT_LVL1_ENTIRES],
    lvl2_normal: [PageBlock; PT_LVL2_ENTIRES],
    lvl2_device: [PageBlock; PT_LVL2_ENTIRES],
}

unsafe impl Sync for InitialPageTable {}
unsafe impl Send for InitialPageTable {}

pub static INITIAL_TT: FakeLock<InitialPageTable> = FakeLock::new(InitialPageTable::default());

impl InitialPageTable {
    pub const fn default() -> Self {
        Self {
            lvl1: [PageTbl::new(); PT_LVL1_ENTIRES],
            lvl2_normal: [PageBlock::new(); PT_LVL2_ENTIRES],
            lvl2_device: [PageBlock::new(); PT_LVL2_ENTIRES],
        }
    }
}

impl PageTable for InitialPageTable {
    fn base(&self) -> VirtAddr {
        VirtAddr::from_raw(&self.lvl1)
    }

    fn entries_per_lvl(&self) -> usize {
        PT_LVL1_ENTIRES
    }

    fn map(
        &mut self,
        _phys: MemRange<PhysAddr>,
        virt: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<(), MmError> {
        let mut size_to_map = usize::from(virt.size());
        let mut curr_addr = usize::from(virt.start());
        let next_lvl = match m_type {
            MappingType::KernelDevice => &mut self.lvl2_device,
            _ => &mut self.lvl2_normal,
        };

        println!(
            "Mapping 0x{:x} -> 0x{:x} as {}",
            curr_addr,
            curr_addr,
            match m_type {
                MappingType::KernelDevice => "device",
                _ => "normal",
            }
        );

        /* Lvl1 addresses 1 GB */
        while {
            let idx = l1_linear_offset(VirtAddr::from(usize::from(curr_addr)));
            let mut new_table = PageTbl::invalid();

            new_table.set_OA(PhysAddr::from((next_lvl as *const _) as u64));
            new_table.valid();

            self.lvl1[idx] = new_table;

            curr_addr += 1 << 30;

            if size_to_map <= 1 << 30 {
                false
            } else {
                size_to_map -= 1 << 30;
                true
            }
        } {}

        let mut size_to_map = usize::from(virt.size());
        let mut curr_addr = usize::from(virt.start());

        /* Lvl2 addresses 2 MB */
        while {
            let idx = l2_linear_offset(VirtAddr::from(curr_addr));
            let mut new_block = PageBlock::invalid();

            new_block.set_OA(curr_addr.into());
            new_block.set_mapping_type(MappingType::KernelRWX);
            new_block.valid();

            next_lvl[idx] = new_block;

            curr_addr += 2 << 20;

            if size_to_map <= 2 << 20 {
                false
            } else {
                size_to_map -= 2 << 20;
                true
            }
        } {}

        Ok(())
    }
}
