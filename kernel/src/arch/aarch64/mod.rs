pub mod backtrace;
pub mod cpuid;
pub mod irq;
pub mod mm;
pub mod regs;
pub mod smp;
pub mod timer;

use core::arch::global_asm;
use core::mem;
use hal::arch::PAGE_SIZE;

pub const PTE_PER_PAGE: usize = PAGE_SIZE / mem::size_of::<usize>();

// TODO: dtb
pub const NUM_CPUS: usize = 2;
pub const PAGE_TABLE_LVLS: u8 = 3;

pub fn init(arg: &loader_protocol::LoaderArg) {
    irq::handlers::set_up_vbar();
    crate::drivers::irq::gic::init(arg);
}

global_asm!(include_str!("boot.s"));
global_asm!(include_str!("context.s"));
