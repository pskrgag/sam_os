use crate::{arch::mm::mmu_flags::*};
use shared::vmm::MappingType;

const KERNEL_DATA: usize = BLOCK_KERNEL_RW | BLOCK_NORMAL_MEM;
const KERNEL_TEXT: usize = BLOCK_KERNEL_RO & !BLOCK_PXN | BLOCK_NORMAL_MEM;
const KERNEL_DATA_RO: usize = BLOCK_KERNEL_RO | BLOCK_NORMAL_MEM;
const KERNEL_RWX: usize = BLOCK_KERNEL_RWX | BLOCK_NORMAL_MEM;
const KERNEL_DEVICE: usize = BLOCK_KERNEL_RW | BLOCK_DEVICE_MEM;

const USER_DATA: usize = BLOCK_USER_RW | BLOCK_NORMAL_MEM;
const USER_DATA_RO: usize = BLOCK_USER_RO | BLOCK_NORMAL_MEM;
const USER_TEXT: usize = BLOCK_USER_RO & !BLOCK_UXN | BLOCK_NORMAL_MEM;

pub fn mapping_type_to_flags(tp: MappingType) -> usize {
    match tp {
        MappingType::KERNEL_DATA => KERNEL_DATA,
        MappingType::KERNEL_TEXT => KERNEL_TEXT,
        MappingType::KERNEL_DATA_RO => KERNEL_DATA_RO,
        MappingType::KERNEL_RWX => BLOCK_KERNEL_RWX,
        MappingType::KERNEL_DEVICE => KERNEL_DEVICE,
        MappingType::USER_DATA => USER_DATA,
        MappingType::USER_DATA_RO => USER_DATA_RO,
        MappingType::USER_TEXT => USER_TEXT,
        _ => panic!(),
    }
}
