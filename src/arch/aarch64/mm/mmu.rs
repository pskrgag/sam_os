// FIXME one day...
#[path = "../qemu/config.rs"]
mod config;

use crate::{
    arch::{
        mm::{initial_map, mmu_flags::*},
        MemoryType, TCR_SZ_SHIFT,
    },
    mm::{
        page_table::{MappingType, PageTable},
        types::*,
    },
};
use core::arch::asm;
use cortex_a::registers::*;
use tock_registers::interfaces::{ReadWriteable, Writeable};

const KERNEL_DATA: u64 = BLOCK_KERNEL_RW | BLOCK_NORMAL_MEM;
const KERNEL_TEXT: u64 = BLOCK_KERNEL_RO & !BLOCK_PXN | BLOCK_NORMAL_MEM;
const KERNEL_DATA_RO: u64 = BLOCK_KERNEL_RO | BLOCK_NORMAL_MEM;
const KERNEL_RWX: u64 = BLOCK_KERNEL_RWX | BLOCK_NORMAL_MEM;
const KERNEL_DEVICE: u64 = KERNEL_DATA | BLOCK_DEVICE_MEM;

pub fn mapping_type_to_flags(tp: MappingType) -> u64 {
    match tp {
        MappingType::KernelData => KERNEL_DATA,
        MappingType::KernelText => KERNEL_TEXT,
        MappingType::KernelDataRo => KERNEL_DATA_RO,
        MappingType::KernelRWX => BLOCK_KERNEL_RWX,
        MappingType::KernelDevice => KERNEL_DEVICE,
    }
}

pub fn init() {
    let tt = initial_map::INITIAL_TT.get();

    for i in &config::MEMORY_LAYOUT {
        let v_range = MemRange::new(i.start.into(), i.size);
        let p_range = MemRange::<PhysAddr>::new(PhysAddr::from(i.start), i.size);

        tt.map(
            p_range,
            v_range,
            if i.tp != MemoryType::MEM {
                MappingType::KernelDevice
            } else {
                MappingType::KernelRWX
            },
        )
        .unwrap();
    }

    TCR_EL1.write(TCR_EL1::TG0::KiB_4 + TCR_EL1::T0SZ.val(64 - TCR_SZ_SHIFT));

    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
    );

    TTBR0_EL1.set_baddr(u64::from(tt.base()));

    unsafe {
        asm!("dsb ishst");
    }

    SCTLR_EL1.modify(SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable + SCTLR_EL1::M::Enable);

    println!("Initial map is set");
}

pub fn init_higher_half() {}
