// FIXME one day...
#[path = "../qemu/config.rs"]
mod config;

use crate::arch::{
    mm::initial_map,
    MemoryType,
};
use crate::mm::types::MemRange;

use tock_registers::interfaces::{Writeable, Readable, ReadWriteable};
use cortex_a::{asm, registers::*};

use core::arch::asm;

pub fn test() {
}

pub fn init() {
    for i in &config::MemoryLayout {
        unsafe { initial_map::initial_tt.populate_indential(&MemRange::new(i.start.into(), i.size), i.tp != MemoryType::MEM); }
    }

    TCR_EL1.write(TCR_EL1::TG0::KiB_4 + TCR_EL1::T0SZ.val(64 - 39));

    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
        MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
    );
    
    println!("{}", TCR_EL1.get());
    unsafe { TTBR0_EL1.set_baddr((&initial_map::initial_tt as *const _) as u64); };
    unsafe { asm!("dsb   sy"); }
    SCTLR_EL1.modify(SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable + SCTLR_EL1::M::Enable);

    println!("Set TTBR0 {:x}", SCTLR_EL1.get());
}
