use crate::arch::{PT_LVL1_ENTIRES, PT_LVL2_ENTIRES, PT_LVL3_ENTIRES};
use rtl::vmm::types::*;

use core::arch::asm;

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

pub unsafe fn set_kernel_page_table(base: PhysAddr) { unsafe {
    asm!("dsb   sy",
         "isb",
         "msr   TTBR1_EL1, {}",
         "isb",
         "tlbi  vmalle1is",
         "dsb   sy",
         "isb",
          in(reg) base.bits(),
    );
}}
