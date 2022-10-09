use crate::mm::types::*;

const INVALID_TTE: usize = 0;
const PTE_WRITE: usize = (0x1 << 6);
const PTE_RO: usize = (0x11 << 6);
const PTE_VALID: usize = 0x11;

#[derive(Copy, Clone)]
pub struct PageBlock(usize);

pub trait PageTable {
    fn map(&self, p: MemRange<PhysAddr>, v: MemRange<VirtAddr>);
    fn unmap(&self, v: MemRange<VirtAddr>);
}

#[inline]
pub fn l1_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) << 4) >> 39
}

#[inline]
pub fn l2_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) << 13) >> 30
}

impl PageBlock {
    pub const fn new() -> Self {
        Self(INVALID_TTE)
    }

    pub const fn valid(mut self) -> Self {
        self.0 |= 0x01;
        self
    }

    pub const fn out_addr(mut self, addr: PhysAddr) -> Self {
        self.0 |= addr.to_pfn() << 21;
        self
    }

    pub const fn write(mut self) -> Self {
        self.0 |= PTE_WRITE;
        self
    }
    
    pub const fn read_only(mut self) -> Self {
        self.0 |= !PTE_WRITE;
        self.0 |= PTE_RO;
        self
    }
}
