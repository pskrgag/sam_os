use crate::{
    arch::{PT_LVL1_ENTIRES, PT_LVL2_ENTIRES, PT_LVL3_ENTIRES},
    mm::types::*,
};

#[inline]
pub fn l1_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 30) & (512 - 1)
}

#[inline]
pub fn l2_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 21) & (512 - 1)
}

#[inline]
pub fn l3_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 12) & (512 - 1)
}
