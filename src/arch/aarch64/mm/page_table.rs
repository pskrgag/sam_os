use crate::{
    mm::types::*,
    arch::{PT_LVL1_ENTIRES, PT_LVL2_ENTIRES},
};

const INVALID_TTE: u64 = 0;
const PTE_WRITE: u64 = 0x00 << 6;
const PTE_RO: u64 = 0x11 << 6;
const PTE_VALID: u64 = 0x11;

#[derive(Copy, Clone)]
pub struct PageBlock(u64);

#[derive(Copy, Clone)]
pub struct PageTbl(u64);

#[inline]
pub fn l1_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 30) & (PT_LVL1_ENTIRES - 1)
}

#[inline]
pub fn l2_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 21) & (PT_LVL2_ENTIRES - 1)
}

impl PageBlock {
    pub const fn new() -> Self {
        Self(INVALID_TTE)
    }

    pub const fn valid(mut self) -> Self {
        self.0 |= 0x01;
        self.0 |= 1 << 10;
        self
    }

    pub const fn out_addr(mut self, addr: PhysAddr) -> Self {
        self.0 |= (addr.to_pfn() as u64) << 12;
        self
    }

    pub const fn write(mut self) -> Self {
        self.0 |= PTE_WRITE;
        self
    }
    
    pub const fn read_only(mut self) -> Self {
        self.0 |= !PTE_WRITE;
        self
    }
    
    pub const fn device(mut self) -> Self {
        self.0 |= 0 << 0x2;
        self.0 |= 1 << 53;
        self
    }
    
    pub const fn normal(mut self) -> Self {
        self.0 = 1 << 0x2;
        self
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

impl PageTbl {
    pub const fn new() -> Self {
        Self(INVALID_TTE)
    }

    pub const fn valid(mut self) -> Self {
        self.0 |= 0b11;
        self
    }

    pub const fn next_lvl(mut self, addr: PhysAddr) -> Self {
        self.0 |= addr.get() as u64;
        self
    }
 
    pub fn get(&self) -> u64 {
        self.0
    }
}
