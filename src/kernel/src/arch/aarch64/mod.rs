#[cfg(feature = "qemu")]
pub mod qemu;
#[cfg(feature = "qemu")]
pub use qemu::config::*;

#[cfg(feature = "orpipc2")]
pub mod orpipc2;
#[cfg(feature = "orpipc2")]
pub use orpipc2::config::*;

#[cfg(all(feature = "qemu", feature = "orpipc2"))]
compile_error!("Please choose one board");

pub mod backtrace;
pub mod context;
pub mod cpuid;
pub mod current;
pub mod irq;
pub mod mm;
pub mod regs;
pub mod smp;
pub mod timer;

use core::arch::global_asm;
use core::mem;
use cortex_a::registers::*;
use tock_registers::interfaces::Readable;

use rtl::arch::PAGE_SIZE;
use rtl::arch::PHYS_OFFSET;

extern crate static_assertions as sa;

pub const PT_LVL1_ENTIRES: usize = PAGE_SIZE / mem::size_of::<u64>();
pub const PT_LVL2_ENTIRES: usize = PAGE_SIZE / mem::size_of::<u64>();
pub const PT_LVL3_ENTIRES: usize = PAGE_SIZE / mem::size_of::<u64>();

pub const TCR_SZ_SHIFT: u64 = 39;

pub const KERNEL_AS_END: usize = usize::MAX;

/// Let it be 126gb
pub const KERNEL_LINEAR_SPACE_SIZE: usize = 10 << 30;
pub const KERNEL_LINEAR_SPACE_BEGIN: usize = PHYS_OFFSET;
pub const KERNEL_LINEAR_SPACE_END: usize = KERNEL_LINEAR_SPACE_BEGIN + KERNEL_LINEAR_SPACE_SIZE;

// TODO: There is memory corruption somewhere. Fix it please
pub const KERNEL_MMIO_BASE: usize = KERNEL_LINEAR_SPACE_END + PAGE_SIZE * 10;

sa::const_assert!(KERNEL_MMIO_BASE > usize::MAX - (1 << (64 - 25)));

// TODO: dtb
pub const NUM_CPUS: usize = 2;

pub const fn user_as_start() -> usize {
    PAGE_SIZE
}

pub const fn user_as_size() -> usize {
    1 << 39
}

pub const PAGE_TABLE_LVLS: u8 = 3;

pub fn time_since_start() -> f64 {
    CNTPCT_EL0.get() as f64 / CNTFRQ_EL0.get() as f64
}

pub fn mmu_on() -> bool {
    (SCTLR_EL1.get() & (1 << 0)) != 0
}

global_asm!(include_str!("boot.s"));
