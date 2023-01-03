use core::arch::asm;
use lock_free_buddy_allocator::cpuid::Cpu;

pub struct CpuLayout;

const MPIDR_HWID_MASK: usize = 0xff00ffffff;

impl Cpu for CpuLayout {
    fn current_cpu() -> usize {
        let mpidr: usize;

        unsafe {
            asm!("mrs   {}, MPIDR_EL1", out(reg) mpidr);
        }

        mpidr & MPIDR_HWID_MASK
    }
}

#[inline]
pub fn current_cpu() -> usize {
    CpuLayout::current_cpu()
}
