use crate::{
    arch::{
        mm::{
            kernel_page_table::{KernelPageTable, KERNEL_PAGE_TABLE},
            mmu_flags::*,
        },
        MemoryType,
        sections::{populate_kernel_sections, KERNEL_SECTIONS},
    },
    kernel::misc::kernel_offset,
    linker_var,
    mm::{
        page_table::{MappingType, PageTable},
        types::*,
        types::MemRange,
    },
};

extern "C" {
    static stext: u64;
    static etext: u64;
    static load_addr: u64;
}

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

fn remap_kernel(table: &mut impl PageTable) {
   let array = KERNEL_SECTIONS.get();

   for i in &*array {
       table.map(None, MemRange::new(VirtAddr::from(i.start()), i.size() as usize), i.mapping_type());
       println!("Mapped {} [0x{:x} -> 0x{:x}]", i.name(), i.start() - kernel_offset(), i.start());
   }
}

pub fn set_up_kernel_tt() {
    let mut table = KERNEL_PAGE_TABLE.lock();
    let new_table = KernelPageTable::new().expect("Failed to allocate kernel tt base");

    *table = new_table;

    populate_kernel_sections();

    remap_kernel(&mut *table);
}
