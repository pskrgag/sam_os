use crate::{
    arch::{self, mm::mmu_flags},
    mm::types::*,
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
            PageTableEntry::from_bits(self.addr
                .to_raw_mut::<usize>()
                .offset(index as isize)
                .read_volatile())
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
                    .to_raw::<u64>()
                    .offset(index as isize)
                    .read_volatile() as usize,
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
        Self::from_bits(arch::mm::mmu_flags::BLOCK_VALID | arch::mm::mmu_flags::BLOCK_ACCESS_FLAG | 0b10)
    }

    pub fn bits(&self) -> usize {
        self.flags
    }
}

pub trait PageTable {
    /* If p is None then caller want linear mapping */
    fn map(
        &mut self,
        _p: Option<MemRange<PhysAddr>>,
        _v: MemRange<VirtAddr>,
        _m_type: MappingType,
    ) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    fn unmap(&mut self, _v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    fn base(&self) -> PhysAddr;
    fn entries_per_lvl(&self) -> usize;
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
