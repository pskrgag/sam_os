use crate::{
    arch::{
        mm::{mmu, mmu_flags},
        PT_LVL1_ENTIRES, PT_LVL2_ENTIRES, PT_LVL3_ENTIRES
    },
    mm::{
        page_table::{MappingType, TranslationTableBlock, TranslationTableTable},
        types::*,
    },
};

#[derive(Copy, Clone, Debug)]
pub struct PageBlock(u64);

#[derive(Copy, Clone, Debug)]
pub struct PageTbl(u64);

#[inline]
pub fn l1_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 30) & (PT_LVL1_ENTIRES - 1)
}

#[inline]
pub fn l2_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 21) & (PT_LVL2_ENTIRES - 1)
}

#[inline]
pub fn l3_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 12) & (PT_LVL3_ENTIRES - 1)
}

impl PageBlock {
    pub const fn new() -> Self {
        Self(0x0)
    }
}

impl PageTbl {
    pub const fn new() -> Self {
        Self(0x0)
    }
}

impl TranslationTableTable for PageTbl {
    type RawTable = u64;

    #[inline]
    fn get(&self) -> u64 {
        self.0
    }

    #[inline]
    fn invalid() -> Self {
        Self(0)
    }

    #[inline]
    fn valid(&mut self) {
        self.0 |= mmu_flags::TABLE_VALID;
    }

    #[inline]
    fn set_OA(&mut self, addr: PhysAddr) {
        self.0 |= addr.get();
    }

    #[inline]
    fn invalidate(&mut self) {
        self.0 |= !mmu_flags::TABLE_VALID;
    }
}

impl TranslationTableBlock for PageBlock {
    type RawBlock = u64;

    #[inline]
    fn invalid() -> Self {
        Self(0)
    }

    #[inline]
    fn set_mapping_type(&mut self, tp: MappingType) {
        self.0 |= mmu::mapping_type_to_flags(tp);
    }

    #[inline]
    fn get(&self) -> Self::RawBlock {
        self.0
    }

    #[inline]
    fn invalidate(&mut self) {
        self.0 |= !mmu_flags::BLOCK_VALID;
    }

    #[inline]
    fn set_OA(&mut self, addr: PhysAddr) {
        self.0 |= u64::from(Pfn::from(addr)) << 12
    }

    #[inline]
    fn valid(&mut self) {
        self.0 |= mmu_flags::BLOCK_VALID;
        self.0 |= mmu_flags::BLOCK_ACCESS_FLAG;
    }
}
