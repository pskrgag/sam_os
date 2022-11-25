use crate::{
    arch::{
        self,
        mm::{self, mmu_flags},
    },
    mm::types::*,
};

use core::ptr;

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

    pub fn is_last(&self) -> bool {
        self.lvl == arch::PAGE_TABLE_LVLS
    }

    pub unsafe fn set_tte(&mut self, index: usize, entry: PageTableEntry) {
        assert!(index < arch::PT_LVL1_ENTIRES);

        self.addr
            .to_raw_mut::<usize>()
            .offset(index as isize)
            .write_volatile(entry.bits());
        // TODO: barriers, please.....
    }

    pub fn index_of(&self, addr: VirtAddr) -> usize {
        match self.lvl {
            1 => arch::mm::page_table::l1_linear_offset(addr),
            2 => arch::mm::page_table::l2_linear_offset(addr),
            3 => arch::mm::page_table::l2_linear_offset(addr),
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
    pub fn new_invalid() -> Self {
        Self::from_bits(0)
    }

    pub fn from_bits(bits: usize) -> Self {
        Self { flags: bits }
    }

    pub fn table() -> Self {
        Self::from_bits(arch::mm::mmu_flags::TABLE_VALID)
    }

    pub fn block() -> Self {
        Self::from_bits(arch::mm::mmu_flags::BLOCK_VALID)
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
    fn invalid() -> Self {
        Self(0)
    }

    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn from_bits(data: usize) -> Self {
        Self(data)
    }

    pub fn set_addr(&mut self, addr: PhysAddr) {
        self.0 | addr.get();
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
