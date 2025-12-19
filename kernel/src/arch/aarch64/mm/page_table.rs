use crate::arch::PTE_PER_PAGE;
use aarch64_cpu::registers::{TTBR0_EL1, Writeable};
use core::arch::asm;
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

pub fn switch_context(pa: PhysAddr) {
    TTBR0_EL1.set(pa.bits() as u64);
    flush_tlb_all();
}

pub fn flush_tlb_all() {
    unsafe {
        asm!("tlbi  vmalle1is");
    }
}
