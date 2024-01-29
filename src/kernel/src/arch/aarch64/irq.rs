use core::arch::asm;

#[inline]
pub unsafe fn enable_all() {
    asm!("msr daifclr, 0x2");
}

#[inline]
pub unsafe fn disable_all() {
    asm!("msr daifset, 0x2");
}

#[inline]
pub fn is_disabled() -> bool {
    let flags: usize;

    unsafe {
        asm!("mrs {}, daif", out(reg) flags);
    }

    flags & (1 << 7) == 0
}

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
    asm!("msr daif, {}", in(reg) flags);
}
