use core::arch::asm;
use rtl::vmm::types::*;

unsafe extern "C" {
    fn __reset();
}

// x1 -- cpu to turn on
// x2 -- entry point
pub unsafe fn boot_cpu(num: usize, ep: usize) { unsafe {
    asm!(
     ".equ PSCI_0_2_FN64_CPU_ON, 0xc4000003",
     "ldr    w0, =PSCI_0_2_FN64_CPU_ON",
     "mov    x3, #0",
     "hvc #0",
     in("x1") num, in("x2") ep, options(nostack)
    );
}}

pub fn bring_up_cpus() {
    unsafe {
        for i in 1..2 {
            boot_cpu(
                i,
                PhysAddr::from(VirtAddr::from(__reset as *const u8 as usize)).get(),
            );
        }
    }
}
