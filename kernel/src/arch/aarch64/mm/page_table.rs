use crate::arch::PTE_PER_PAGE;
use hal::address::*;

#[inline]
pub fn l0_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 39) & (PTE_PER_PAGE - 1)
}

#[inline]
pub fn l1_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 30) & (PTE_PER_PAGE - 1)
}

#[inline]
pub fn l2_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 21) & (PTE_PER_PAGE - 1)
}

#[inline]
pub fn l3_linear_offset(va: VirtAddr) -> usize {
    (usize::from(va) >> 12) & (PTE_PER_PAGE - 1)
}
