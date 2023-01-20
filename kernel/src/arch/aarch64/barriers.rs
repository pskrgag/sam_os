use core::arch::asm;

pub unsafe fn smp_mb() {
    asm!("dmb   ish");
}

pub unsafe fn smp_rb() {
    asm!("dmb   ishld");
}

pub unsafe fn smp_wb() {
    asm!("dmb   ishst");
}

pub unsafe fn mb() {
    asm!("dsb");
}

pub unsafe fn wb() {
    asm!("dsb   st");
}

pub unsafe fn isb() {
    asm!("isb");
}
