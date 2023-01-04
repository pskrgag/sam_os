#[cfg(feature = "qemu")]
pub mod qemu;

#[cfg(feature = "qemu")]
pub use qemu::config::*;

pub mod backtrace;
pub mod context;
pub mod cpuid;
pub mod interrupts;
pub mod irq;
pub mod mm;
pub mod regs;
pub mod smp;

use crate::kernel::misc::genmask;
use core::mem;
use cortex_a::registers::*;
use tock_registers::interfaces::Readable;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PT_LVL1_ENTIRES: usize = PAGE_SIZE / mem::size_of::<u64>();
pub const PT_LVL2_ENTIRES: usize = PAGE_SIZE / mem::size_of::<u64>();
pub const PT_LVL3_ENTIRES: usize = PAGE_SIZE / mem::size_of::<u64>();

pub const TCR_SZ_SHIFT: u64 = 39;

pub const KERNEL_AS_END: usize = usize::MAX;

// TODO: dtb
pub const NUM_CPUS: usize = 2;

pub fn kernel_as_start() -> usize {
    genmask(63, 64 - 39)
}

pub fn kernel_as_size() -> usize {
    KERNEL_AS_END - kernel_as_start()
}

pub const fn user_as_start() -> usize {
    PAGE_SIZE
}

pub const fn user_as_size() -> usize {
    1 << 39
}

pub const PHYS_OFFSET: usize = 0xffffffff80000000;

pub const PAGE_TABLE_LVLS: u8 = 3;

pub fn time_since_start() -> f64 {
    CNTPCT_EL0.get() as f64 / CNTFRQ_EL0.get() as f64
}

pub fn mmu_on() -> bool {
    (SCTLR_EL1.get() & (1 << 0)) != 0
}
