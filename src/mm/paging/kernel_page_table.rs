use crate::{
    arch::PT_LVL1_ENTIRES,
    arch::{self, mm::mmu, PAGE_SIZE},
    kernel::{
        locking::spinlock::{Spinlock, SpinlockGuard},
        misc::*,
    },
    mm::{
        allocators::page_alloc::PAGE_ALLOC,
        paging::page_table::{
            MappingType, MmError, PageFlags, PageTable, PageTableBlock, PageTableEntry,
        },
        types::{MemRange, PhysAddr, VirtAddr},
    },
};

pub struct KernelPageTable {
    base: VirtAddr,
}

pub static KERNEL_PAGE_TABLE: Spinlock<KernelPageTable> = Spinlock::new(KernelPageTable::default());

impl KernelPageTable {
    pub const fn default() -> Self {
        Self {
            base: VirtAddr::new(0_usize),
        }
    }

    pub fn new() -> Option<Self> {
        let base = PAGE_ALLOC.lock().alloc_pages(1)?;
        let mut new_table = Self {
            base: VirtAddr::from(base),
        };

        let base_va = new_table.base;
        println!("Base 0x{:x} 0x{:x}", base.get(), base_va.get());

        // unsafe {
        //     core::slice::from_raw_parts_mut(base_va.to_raw_mut::<u8>(), PAGE_SIZE).fill(0);
        // }

        new_table
            .map(
                None,
                MemRange::new(base_va, PAGE_SIZE),
                MappingType::KernelRWX,
            )
            .ok()?;

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
        p: Option<MemRange<PhysAddr>>,
        v: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<(), MmError> {
        let flags = mmu::mapping_type_to_flags(m_type);
        let mut lvl1_sz = v.size();
        let mut va = v.start();
        let pa = if let Some(range) = p {
            Some(range.start())
        } else {
            None
        };

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

                    self.map(
                        None,
                        MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                        MappingType::KernelRWX,
                    )?;

                    PageTableBlock::new(VirtAddr::from(new_page), 2)
                }
            };
            let mut lvl2_sz = if lvl1_sz > _1GB { _1GB } else { lvl1_sz };
            assert!(table_block_1.lvl() == 1);

            while {
                let mut lvl3_sz = if lvl2_sz > _2MB { _2MB } else { lvl2_sz };
                let lvl2_index = table_block_2.index_of(va);
                let mut table_block_3 = match table_block_2.next(lvl2_index) {
                    Some(e) => e,
                    None => {
                        let new_page = PAGE_ALLOC
                            .lock()
                            .alloc_pages(1)
                            .expect("Failed to allocate memory");
                        let new_entry =
                            PageTableEntry::from_bits(PageFlags::table().bits() | new_page.get());

                        unsafe { table_block_2.set_tte(lvl2_index, new_entry) };

                        self.map(
                            None,
                            MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                            MappingType::KernelRWX,
                        )?;

                        PageTableBlock::new(VirtAddr::from(new_page), 3)
                    }
                };

                assert!(table_block_2.lvl() == 2);

                while {
                    let lvl3_index: usize = table_block_3.index_of(va);
                    let ao = if let Some(addr) = pa {
                        addr.get()
                    } else {
                        PhysAddr::from(va).get()
                    };
                    println!("Here 0x{:x}", ao);

                    assert!(!table_block_3.tte(lvl3_index).valid());

                    unsafe {
                        table_block_3.set_tte(
                            lvl3_index,
                            PageTableEntry::from_bits(PageFlags::block().bits() | flags | ao),
                        );
                    };

                    assert!(table_block_3.lvl() == 3);

                    // println!(
                    //     "Index 3 {} 0x{:x} 0x{:x}",
                    //     lvl3_index,
                    //     va.get(),
                    //     PhysAddr::from(table_block_3.addr()).get(),
                    // );

                    va.add(arch::PAGE_SIZE);

                    if pa.is_some() {
                        pa.unwrap().add(arch::PAGE_SIZE);
                    }

                    lvl3_sz -= _4KB;
                    lvl3_sz != 0
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

    pub fn table_walk(&self, va: VirtAddr) -> Option<()> {
        println!("Doing page table walk for 0x{:x}", va.get());
        let table_block_1 = self.lvl1();
        let lvl1_index = table_block_1.index_of(va);
        let lvl2_tte = table_block_1.tte(lvl1_index);

        println!(
            "Lvl1 PA 0x{:x} tte 0x{:x}",
            lvl2_tte.addr().get(),
            lvl2_tte.bits()
        );

        let table_block_2 = table_block_1.next(lvl1_index)?;
        let lvl2_index = table_block_2.index_of(va);
        let lvl3_tte = table_block_2.tte(lvl2_index);

        println!(
            "Lvl2 PA 0x{:x} tte 0x{:x}",
            lvl3_tte.addr().get(),
            lvl3_tte.bits()
        );

        let table_block_3 = table_block_2.next(lvl2_index)?;
        let lvl3_index = table_block_3.index_of(va);

        println!("Lvl3 tte 0x{:x}", table_block_3.tte(lvl3_index).bits());

        Some(())
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
