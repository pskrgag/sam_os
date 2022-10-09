use core::arch::{global_asm, asm};

extern "C" {
    static exteption_vector: usize;
}

#[inline]
pub fn set_up_vbar() {
    unsafe {
         asm!("msr VBAR_EL1, {}", in(reg) &exteption_vector);
    }
}

#[no_mangle]
pub extern "C" fn kern_sync64(esr_el1: usize, far_el1: usize) -> ! {
    println!("Kernel synch expection\nESR_EL1 0x{:x} FAR_EL1 0x{:x}\n", esr_el1, far_el1);

    panic!();
}

#[no_mangle]
pub extern "C" fn kern_exception_bug(esr_el1: usize) -> ! {
    println!("Something weird happened");
    println!("No idea how to deal with 0x{:x}", esr_el1);

    panic!();
}

global_asm!(include_str!("interrupts.S"));
