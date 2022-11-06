#[cfg(feature = "qemu")]
pub mod qemu;

#[cfg(feature = "qemu")]
pub use qemu::config::*;

pub mod interrupts;
pub mod sections;
pub mod mm;

use core::mem;
use cortex_a::registers::*;
use mm::page_table::PageBlock;
use tock_registers::interfaces::Readable;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PT_LVL1_ENTIRES: usize = PAGE_SIZE / mem::size_of::<PageBlock>();
pub const PT_LVL2_ENTIRES: usize = PAGE_SIZE / mem::size_of::<PageBlock>();
pub const PT_LVL3_ENTIRES: usize = PAGE_SIZE / mem::size_of::<PageBlock>();

pub const TCR_SZ_SHIFT: u64 = 39;

pub fn time_since_start() -> f64 {
    CNTPCT_EL0.get() as f64 / CNTFRQ_EL0.get() as f64
}
