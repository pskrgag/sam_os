use crate::{
    arch::PT_LVL1_ENTIRES,
    arch::{
        self,
        mm::{
            mmu,
            page_table::{l1_linear_offset, l2_linear_offset},
        },
        PAGE_SIZE,
    },
    kernel::{
        locking::spinlock::{Spinlock, SpinlockGuard},
        misc::*,
    },
    mm::{
        page_alloc::PAGE_ALLOC,
        paging::page_table::{
            MappingType, MmError, PageFlags, PageTable, PageTableBlock, PageTableEntry,
        },
        phys_to_virt_linear,
        types::{MemRange, PhysAddr, VirtAddr},
    },
};

use alloc::boxed::Box;
use core::pin::Pin;

pub struct KernelPageTable {
    base: VirtAddr,
}

static KERNEL_PAGE_TABLE: Spinlock<KernelPageTable> = Spinlock::new(KernelPageTable::default());

impl KernelPageTable {
    pub const fn default() -> Self {
        Self {
            base: VirtAddr::new(0_usize),
        }
    }

    pub fn new() -> Option<Self> {
        let base = PAGE_ALLOC.lock().alloc_pages(1);
        let new_table = match base {
            Some(b) => Some(Self { base: b.into() }),
            None => None,
        };

        if new_table.is_none() {
            return None;
        }

        let mut new_table = new_table.unwrap();
        let base_va = phys_to_virt_linear(new_table.base());

        new_table.map(
            None,
            MemRange::new(base_va, PAGE_SIZE),
            MappingType::KernelData,
        );

        unsafe {
            core::slice::from_raw_parts_mut(base_va.to_raw_mut::<u8>(), PAGE_SIZE).fill(0);
        }

        Some(new_table)
    }
}

impl PageTable for KernelPageTable {
    #[inline]
    fn base(&self) -> PhysAddr {
        PhysAddr::from(self.base)
    }

    #[inline]
    fn entries_per_lvl(&self) -> usize {
        PT_LVL1_ENTIRES
    }

    // TODO: Hugetables?
    fn map(
        &mut self,
        _p: Option<MemRange<PhysAddr>>,
        v: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<(), MmError> {
        let flags = mmu::mapping_type_to_flags(m_type);
        let mut lvl1_sz = v.size();
        let mut va = v.start();

        /* Lvl1 loop */
        while {
            let mut table_block_1 = self.lvl1();
            let lvl1_index = table_block_1.index_of(va);
            let mut table_block_2 = match table_block_1.next(lvl1_index) {
                Some(e) => e,
                None => {
                    let new_page = PAGE_ALLOC
                        .lock()
                        .alloc_pages(1)
                        .expect("Failed to allocate memory");
                    let new_entry =
                        PageTableEntry::from_bits(PageFlags::table().bits() | new_page.get());

                    unsafe { table_block_1.set_tte(lvl1_index, new_entry) };

                    PageTableBlock::new(VirtAddr::from(new_page), 1)
                }
            };
            let mut lvl2_sz = if lvl1_sz > _1GB { _1GB } else { lvl1_sz };

            while {
                let mut lvl2_sz = if lvl2_sz > _2MB { _2MB } else { lvl2_sz };
                let lvl2_index = table_block_1.index_of(va);
                let mut table_block_3 = match table_block_2.next(lvl2_index) {
                    Some(e) => e,
                    None => {
                        let new_page = PAGE_ALLOC
                            .lock()
                            .alloc_pages(1)
                            .expect("Failed to allocate memory");
                        let new_entry =
                            PageTableEntry::from_bits(PageFlags::table().bits() | new_page.get());

                        unsafe { table_block_1.set_tte(lvl1_index, new_entry) };

                        PageTableBlock::new(VirtAddr::from(new_page), 2)
                    }
                };
                while {
                    let lvl3_index: usize = table_block_3.index_of(va);
                    unsafe { table_block_3.set_tte(lvl3_index, PageTableEntry::from_bits(flags | PhysAddr::from(va).get())) };

                    va.add(arch::PAGE_SIZE);
                    lvl1_sz -= _4KB;
                    lvl1_sz != 0
                } {}

                if lvl2_sz < _2MB {
                    return Ok(());
                }

                lvl2_sz -= _2MB;
                lvl2_sz != 0
            } {}

            if lvl1_sz < _1GB {
                return Ok(());
            }
            lvl1_sz -= _1GB;
            lvl1_sz != 0
        } {}

        todo!();
    }
}

impl KernelPageTable {
    fn lvl1(&self) -> PageTableBlock {
        PageTableBlock::new(self.base, 1)
    }
}

pub fn init() {
    let mut table = KERNEL_PAGE_TABLE.lock();
    *table = KernelPageTable::new().expect("Failed to allocate tt base");

    println!("Allocated kernel page table base");
}

pub fn kernel_page_table() -> SpinlockGuard<'static, KernelPageTable> {
    KERNEL_PAGE_TABLE.lock()
}
