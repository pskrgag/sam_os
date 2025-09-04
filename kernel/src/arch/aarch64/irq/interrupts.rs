use core::arch::asm;

#[inline]
pub unsafe fn enable_all() {
    unsafe {
        asm!("msr daifclr, 0x2");
    }
}

#[inline]
pub unsafe fn disable_all() {
    unsafe {
        asm!("msr daifset, 0x2");
    }
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
    unsafe {
        asm!("msr daif, {}", in(reg) flags);
    }
}
