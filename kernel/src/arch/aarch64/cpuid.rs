use core::arch::asm;

const MPIDR_HWID_MASK: usize = 0xff00ffffff;

#[inline]
pub fn current_cpu() -> usize {
    let mpidr: usize;

    unsafe {
        asm!("mrs   {}, MPIDR_EL1", out(reg) mpidr);
    }

    mpidr & MPIDR_HWID_MASK
}
