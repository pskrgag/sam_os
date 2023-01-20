use crate::drivers::irq::irq_dispatch;
use crate::kernel::sched;
use crate::kernel::syscalls::do_syscall;
use core::arch::{asm, global_asm};
use core::fmt;

global_asm!(include_str!("interrupts.S"));
global_asm!(include_str!("boot.s"));

extern "C" {
    static exteption_vector: u64;
}

pub struct ExceptionCtx {
    pub x0: usize,
    pub x1: usize,
    pub x2: usize,
    pub x3: usize,
    pub x4: usize,
    pub x5: usize,
    pub x6: usize,
    pub x7: usize,
    pub x8: usize,
    pub x9: usize,
    pub x10: usize,
    pub x11: usize,
    pub x12: usize,
    pub x13: usize,
    pub x14: usize,
    pub x15: usize,
    pub x16: usize,
    pub x17: usize,
    pub x18: usize,
    pub x19: usize,
    pub x20: usize,
    pub x21: usize,
    pub x22: usize,
    pub x23: usize,
    pub x24: usize,
    pub x25: usize,
    pub x26: usize,
    pub x27: usize,
    pub x28: usize,
    pub x29: usize,
}

impl fmt::Display for ExceptionCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "x0  =  0x{:x}\n \
        x1  =  0x{:x}\n \
        x2  =  0x{:x}\n \
        x3  =  0x{:x}\n \
        x4  =  0x{:x}\n \
        x5  =  0x{:x}\n \
        x6  =  0x{:x}\n \
        x7  =  0x{:x}\n \
        x8  =  0x{:x}\n \
        x9  =  0x{:x}\n \
        x10 = 0x{:x}\n \
        x11 = 0x{:x}\n \
        x12 = 0x{:x}\n \
        x13 = 0x{:x}\n \
        x14 = 0x{:x}\n \
        x15 = 0x{:x}\n \
        x16 = 0x{:x}\n \
        x17 = 0x{:x}\n \
        x18 = 0x{:x}\n \
        x19 = 0x{:x}\n \
        x20 = 0x{:x}\n \
        x21 = 0x{:x}\n \
        x22 = 0x{:x}\n \
        x23 = 0x{:x}\n \
        x24 = 0x{:x}\n \
        x25 = 0x{:x}\n \
        x26 = 0x{:x}\n \
        x27 = 0x{:x}\n \
        x28 = 0x{:x}\n \
        x29 = 0x{:x}\n",
            self.x0,
            self.x1,
            self.x2,
            self.x3,
            self.x4,
            self.x5,
            self.x6,
            self.x7,
            self.x8,
            self.x9,
            self.x10,
            self.x11,
            self.x12,
            self.x13,
            self.x14,
            self.x15,
            self.x16,
            self.x17,
            self.x18,
            self.x19,
            self.x20,
            self.x21,
            self.x22,
            self.x23,
            self.x24,
            self.x25,
            self.x26,
            self.x27,
            self.x28,
            self.x29,
        )
    }
}

#[inline]
pub fn set_up_vbar() {
    unsafe {
        asm!("msr VBAR_EL1, {}",
             "isb",
            in(reg) &exteption_vector);
    }
}

#[no_mangle]
pub extern "C" fn kern_sync64(
    esr_el1: usize,
    far_el1: usize,
    elr_el1: usize,
    dump: &ExceptionCtx,
) -> ! {
    println!("!!! Kernel sync exception");
    println!("{}", dump);
    println!(
        "ESR_EL1 0x{:x} FAR_EL1 0x{:x}, ELR_EL1 0x{:x}",
        esr_el1, far_el1, elr_el1
    );

    panic!("Unhandler kernel sync exception");
}

#[no_mangle]
pub extern "C" fn kern_irq() {
    irq_dispatch();

    unsafe {
        sched::run();
    }
}

#[no_mangle]
pub extern "C" fn kern_exception_bug(esr_el1: usize, far_el1: usize, elr_el1: usize) -> ! {
    println!("Something weird happened");
    println!(
        "ESR_EL1 0x{:x} FAR_EL1 0x{:x}, ELR_EL1 0x{:x}",
        esr_el1, far_el1, elr_el1
    );
    println!("No idea how to deal with 0x{:x}", esr_el1);

    panic!();
}

#[no_mangle]
pub extern "C" fn user_sync(esr_el1: usize, elr_el1: usize) {
    println!(
        "!!! Kernel sync from EL0    ESR_EL1 0x{:x}    ELR_EL1 0x{:x}",
        esr_el1, elr_el1
    );

    panic!("Some user thread has panicked! No idea how to deal with it");
}

#[no_mangle]
pub extern "C" fn user_syscall(
    x0: usize,
    x1: usize,
    x2: usize,
    x3: usize,
    x4: usize,
    x5: usize,
) -> usize {
    println!("User syscall {}", x0);

    do_syscall(x0, x1, x2, x3, x4, x5)
}
