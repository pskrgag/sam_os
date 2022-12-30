use crate::{
    arch::PT_LVL1_ENTIRES,
    arch::{self, mm::mmu_flags},
    arch::{mm::mmu, PAGE_SIZE},
    kernel::misc::*,
    mm::paging::kernel_page_table::kernel_page_table,
    mm::{allocators::page_alloc::page_allocator, types::*},
};

#[derive(Debug)]
pub enum MmError {
    InvalidAddr,
    NoMem,
    NotImpl,
}

#[derive(Clone, Copy)]
pub enum MappingType {
    KernelData,
    KernelText,
    KernelDataRo,
    KernelRWX,
    KernelDevice,
    KernelNothing,

    UserData,
    UserText,
    UserDataRo,
}

pub struct PageFlags {
    flags: usize,
}

pub struct PageTableBlock {
    addr: VirtAddr,
    lvl: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct PageTableEntry(usize);

pub struct PageTable {
    base: VirtAddr,
    kernel: bool,
}

impl PageTableBlock {
    pub fn new(addr: VirtAddr, lvl: u8) -> Self {
        Self {
            addr: addr,
            lvl: lvl,
        }
    }

    pub fn addr(&self) -> VirtAddr {
        self.addr
    }

    pub fn lvl(&self) -> u8 {
        self.lvl
    }

    pub fn is_last(&self) -> bool {
        self.lvl == arch::PAGE_TABLE_LVLS
    }

    pub unsafe fn set_tte(&mut self, index: usize, entry: PageTableEntry) {
        assert!(index < 512);

        self.addr
            .to_raw_mut::<usize>()
            .offset(index as isize)
            .write_volatile(entry.bits());
        // TODO: barriers, please.....
    }

    pub fn tte(&self, index: usize) -> PageTableEntry {
        unsafe {
            PageTableEntry::from_bits(
                self.addr
                    .to_raw_mut::<usize>()
                    .offset(index as isize)
                    .read_volatile(),
            )
        }
    }

    pub fn index_of(&self, addr: VirtAddr) -> usize {
        match self.lvl {
            1 => arch::mm::page_table::l1_linear_offset(addr),
            2 => arch::mm::page_table::l2_linear_offset(addr),
            3 => arch::mm::page_table::l3_linear_offset(addr),
            _ => panic!("Wrong page table block index"),
        }
    }

    pub fn next(&self, index: usize) -> Option<Self> {
        assert!(!self.is_last());

        let entry_next = unsafe {
            PageTableEntry::from_bits(
                self.addr
                    .to_raw::<usize>()
                    .offset(index as isize)
                    .read_volatile(),
            )
        };

        if entry_next.valid() {
            Some(Self::new(VirtAddr::from(entry_next.addr()), self.lvl + 1))
        } else {
            None
        }
    }
}

impl PageFlags {
    pub fn from_bits(bits: usize) -> Self {
        Self { flags: bits }
    }

    pub fn table() -> Self {
        Self::from_bits(arch::mm::mmu_flags::TABLE_VALID)
    }

    pub fn block() -> Self {
        Self::from_bits(
            arch::mm::mmu_flags::BLOCK_VALID | arch::mm::mmu_flags::BLOCK_ACCESS_FLAG | 0b10,
        )
    }

    pub fn bits(&self) -> usize {
        self.flags
    }
}

impl PageTable {
    pub const fn default(kernel: bool) -> Self {
        Self {
            base: VirtAddr::new(0_usize),
            kernel: kernel,
        }
    }

    pub fn new(kernel: bool) -> Option<Self> {
        let base: PhysAddr = page_allocator().alloc(1)?.into();
        let mut new_table = Self {
            base: VirtAddr::from(base),
            kernel: kernel,
        };

        let base_va = new_table.base;

        // unsafe {
        //     core::slice::from_raw_parts_mut(base_va.to_raw_mut::<u8>(), PAGE_SIZE).fill(0);
        // }

        println!("PAge table base 0x{:x}", base_va.get());
        if kernel {
            new_table
                .map(
                    None,
                    MemRange::new(base_va, PAGE_SIZE),
                    MappingType::KernelRWX,
                )
                .ok()?;
        } else {
            kernel_page_table()
                .map(
                    None,
                    MemRange::new(base_va, PAGE_SIZE),
                    MappingType::KernelRWX,
                )
                .ok()?;
        }

        Some(new_table)
    }

    /* If p is None then caller want linear mapping */
    pub fn map(
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
                    let new_page: PhysAddr = page_allocator()
                        .alloc(1)
                        .expect("Failed to allocate memory")
                        .into();
                    let new_entry =
                        PageTableEntry::from_bits(PageFlags::table().bits() | new_page.get());

                    unsafe { table_block_1.set_tte(lvl1_index, new_entry) };

                    if !self.kernel {
                        kernel_page_table().map(
                            None,
                            MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                            MappingType::KernelData,
                        )?;
                    }

                    self.map(
                        None,
                        MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                        MappingType::KernelData,
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
                        let new_page: PhysAddr = page_allocator()
                            .alloc(1)
                            .expect("Failed to allocate memory")
                            .into();
                        let new_entry =
                            PageTableEntry::from_bits(PageFlags::table().bits() | new_page.get());

                        unsafe { table_block_2.set_tte(lvl2_index, new_entry) };

                        if !self.kernel {
                            kernel_page_table().map(
                                None,
                                MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                                MappingType::KernelData,
                            )?;
                        }

                        self.map(
                            None,
                            MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                            MappingType::KernelData,
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

                    assert!(!table_block_3.tte(lvl3_index).valid());

                    unsafe {
                        table_block_3.set_tte(
                            lvl3_index,
                            PageTableEntry::from_bits(PageFlags::block().bits() | flags | ao),
                        );
                    };

                    assert!(table_block_3.lvl() == 3);

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

        unreachable!();
    }

    pub fn unmap(&mut self, _v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    #[inline]
    pub fn base(&self) -> PhysAddr {
        PhysAddr::from(self.base)
    }

    #[inline]
    fn entries_per_lvl(&self) -> usize {
        PT_LVL1_ENTIRES
    }

    #[inline]
    fn lvl1(&self) -> PageTableBlock {
        PageTableBlock::new(self.base, 1)
    }
}

impl PageTableEntry {
    pub fn valid_block() -> Self {
        Self(mmu_flags::BLOCK_ACCESS_FLAG | mmu_flags::BLOCK_VALID)
    }

    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn from_bits(data: usize) -> Self {
        Self(data)
    }

    pub fn and(&mut self, data: usize) -> &mut Self {
        self.0 |= data;
        self
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & !mmu_flags::PAGE_ENTRY_FLAGS_MASK)
    }

    pub fn flags(&self) -> PageFlags {
        PageFlags::from_bits(self.0 & mmu_flags::PAGE_ENTRY_FLAGS_MASK)
    }

    pub fn valid(&self) -> bool {
        self.0 & 0b11 != 0
    }
}