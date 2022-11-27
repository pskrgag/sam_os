use core::arch::{asm, global_asm};

global_asm!(include_str!("interrupts.S"));
global_asm!(include_str!("boot.s"));

extern "C" {
    static exteption_vector: u64;
}

struct Esr(u64);

#[inline]
pub fn set_up_vbar() {
    unsafe {
        asm!("msr VBAR_EL1, {}", in(reg) &exteption_vector);
    }
}

#[no_mangle]
pub extern "C" fn kern_sync64(esr_el1: usize, far_el1: usize) -> ! {
    println!("!!! Kernel sync exception");
    println!("ESR_EL1 0x{:x} FAR_EL1 0x{:x}", esr_el1, far_el1);

    panic!("Unhandler kernel sync exception");
}

#[no_mangle]
pub extern "C" fn kern_irq(esr_el1: usize, far_el1: usize) -> ! {
    println!("!!! Kernel irq");

    panic!("Unhandler kernel sync exception");
}

#[no_mangle]
pub extern "C" fn kern_exception_bug(esr_el1: usize) -> ! {
    println!("Something weird happened");
    println!("No idea how to deal with 0x{:x}", esr_el1);

    panic!();
}
