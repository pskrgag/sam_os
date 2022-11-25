use crate::{
    arch::{mm::mmu_flags::*, MemoryType},
    linker_var,
    mm::{paging::page_table::MappingType, types::MemRange, types::*},
};

extern "C" {
    static stext: u64;
    static etext: u64;
    static load_addr: u64;
}

const KERNEL_DATA: usize = BLOCK_KERNEL_RW | BLOCK_NORMAL_MEM;
const KERNEL_TEXT: usize = BLOCK_KERNEL_RO & !BLOCK_PXN | BLOCK_NORMAL_MEM;
const KERNEL_DATA_RO: usize = BLOCK_KERNEL_RO | BLOCK_NORMAL_MEM;
const KERNEL_RWX: usize = BLOCK_KERNEL_RWX | BLOCK_NORMAL_MEM;
const KERNEL_DEVICE: usize = KERNEL_DATA | BLOCK_DEVICE_MEM;

pub fn mapping_type_to_flags(tp: MappingType) -> usize {
    match tp {
        MappingType::KernelData => KERNEL_DATA,
        MappingType::KernelText => KERNEL_TEXT,
        MappingType::KernelDataRo => KERNEL_DATA_RO,
        MappingType::KernelRWX => BLOCK_KERNEL_RWX,
        MappingType::KernelDevice => KERNEL_DEVICE,
    }
}
