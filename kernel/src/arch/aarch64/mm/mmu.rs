use crate::arch::mm::mmu_flags::*;
use rtl::vmm::types::*;
use rtl::vmm::*;

const KERNEL_DATA: usize = BLOCK_KERNEL_RW | BLOCK_NORMAL_MEM;
const KERNEL_TEXT: usize = BLOCK_KERNEL_RO & !BLOCK_PXN | BLOCK_NORMAL_MEM;
const KERNEL_DATA_RO: usize = BLOCK_KERNEL_RO | BLOCK_NORMAL_MEM;
const KERNEL_RWX: usize = BLOCK_KERNEL_RWX | BLOCK_NORMAL_MEM;
const KERNEL_DEVICE: usize = BLOCK_KERNEL_RW | BLOCK_DEVICE_MEM;

const USER_DATA: usize = BLOCK_USER_RW | BLOCK_NORMAL_MEM;
const USER_DATA_RO: usize = BLOCK_USER_RO | BLOCK_NORMAL_MEM;
const USER_TEXT: usize = BLOCK_USER_RO & !BLOCK_UXN | BLOCK_NORMAL_MEM;
const USER_RWX: usize = BLOCK_USER_RWX | BLOCK_NORMAL_MEM;
const USER_DEVICE: usize = BLOCK_USER_RW | BLOCK_DEVICE_MEM;

pub fn mapping_type_to_flags(tp: MappingType, user_mode: bool) -> usize {
    if user_mode {
        match tp {
            MappingType::Data => USER_DATA,
            MappingType::RoData => USER_DATA_RO,
            MappingType::Text => USER_TEXT,
            MappingType::Rwx => USER_RWX,
            MappingType::Device => USER_DEVICE,
            MappingType::None => 0,
        }
    } else {
        match tp {
            MappingType::Rwx => KERNEL_RWX,
            MappingType::Text => KERNEL_TEXT,
            MappingType::Data => KERNEL_DATA,
            MappingType::RoData => KERNEL_DATA_RO,
            MappingType::Device => KERNEL_DEVICE,
            MappingType::None => 0,
        }
    }
}

// TODO: less aggressive tlb maintainence (i.e last level flush with ASID support)
pub unsafe fn flush_tlb_page_last(_v: VirtAddr) {
    unsafe {
        core::arch::asm!("tlbi       vmalle1");
    }
}
