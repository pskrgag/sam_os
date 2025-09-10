use core::arch::global_asm;

pub mod mmu;

global_asm!(include_str!("boot.s"));
