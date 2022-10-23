// FIXME one day...
#[path = "../qemu/config.rs"]
mod config;

use crate::{
    arch::{
        mm::initial_map,
        MemoryType,
        TCR_SZ_SHIFT,
    },
    mm::{
        page_table::PageTable,
        types::MemRange,
    },
};
use tock_registers::interfaces::{Writeable, Readable, ReadWriteable};
use cortex_a::registers::*;
use core::arch::asm;

pub fn init() {
    let tt = initial_map::INITIAL_TT.get();

    for i in &config::MEMORY_LAYOUT {
        tt.populate_indential(&MemRange::new(i.start.into(), i.size), i.tp != MemoryType::MEM);
    }

    TCR_EL1.write(TCR_EL1::TG0::KiB_4 + TCR_EL1::T0SZ.val(64 - TCR_SZ_SHIFT));

    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
        MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
    );
    
    TTBR0_EL1.set_baddr(u64::from(tt.lvl1()));
    unsafe { asm!("dsb ishst"); }
    SCTLR_EL1.modify(SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable + SCTLR_EL1::M::Enable);

    println!("Initial map is set");
}

pub fn init_higher_half() {

}
