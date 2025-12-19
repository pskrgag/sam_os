use aarch64_cpu::registers::{MPIDR_EL1, Readable};

const MPIDR_HWID_MASK: usize = 0xff00ffffff;

#[inline]
pub fn current_cpu() -> usize {
    let mpidr = MPIDR_EL1.get() as usize;

    mpidr & MPIDR_HWID_MASK
}
