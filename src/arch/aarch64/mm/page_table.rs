use crate::{
    arch::{
        mm::{mmu, mmu_flags},
        PT_LVL1_ENTIRES, PT_LVL2_ENTIRES, PT_LVL3_ENTIRES,
    },
    mm::types::*,
};

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
