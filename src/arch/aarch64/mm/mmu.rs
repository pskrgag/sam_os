use crate::{arch::mm::mmu_flags::*, mm::paging::page_table::MappingType};

const KERNEL_DATA: usize = BLOCK_KERNEL_RW | BLOCK_NORMAL_MEM;
const KERNEL_TEXT: usize = BLOCK_KERNEL_RO & !BLOCK_PXN | BLOCK_NORMAL_MEM;
const KERNEL_DATA_RO: usize = BLOCK_KERNEL_RO | BLOCK_NORMAL_MEM;
const KERNEL_RWX: usize = BLOCK_KERNEL_RWX | BLOCK_NORMAL_MEM;
const KERNEL_DEVICE: usize = BLOCK_KERNEL_RW | BLOCK_DEVICE_MEM;

pub fn mapping_type_to_flags(tp: MappingType) -> usize {
    match tp {
        MappingType::KernelData => KERNEL_DATA,
        MappingType::KernelText => KERNEL_TEXT,
        MappingType::KernelDataRo => KERNEL_DATA_RO,
        MappingType::KernelRWX => BLOCK_KERNEL_RWX,
        MappingType::KernelDevice => KERNEL_DEVICE,
        MappingType::KernelNothing => todo!(),
    }
}
