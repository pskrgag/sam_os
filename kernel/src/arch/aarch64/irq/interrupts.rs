use core::arch::asm;

#[inline]
pub fn get_flags() -> usize {
    let flags: usize;

    unsafe {
        asm!("mrs {}, daif", out(reg) flags);
    }

    flags
}

#[inline]
pub unsafe fn set_flags(flags: usize) {
    unsafe {
        asm!("msr daif, {}", in(reg) flags);
    }
}
