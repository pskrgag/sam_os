use core::arch::asm;

#[inline]
pub unsafe fn enable_all() {
    asm!("msr daifclr, 0x2");
}

#[inline]
pub unsafe fn disable_all() {
    asm!("msr daifset, 0x2");
}
