use crate::arch;
use crate::arch::mm::page_table::PageBlock;
use core::mem;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = (1 << PAGE_SHIFT);
pub const UART_BASE: *mut u8 = 0x0900_0000 as *mut u8;
pub const PT_LVL1_ENTIRES: usize = PAGE_SIZE / mem::size_of::<PageBlock>();
pub const PT_LVL2_ENTIRES: usize = PAGE_SIZE / mem::size_of::<PageBlock>();

pub const MemoryLayout: [arch::MemoryRegion; 2] = [
    arch::MemoryRegion {
        start: 0x09000000,
        size: 0x00001000,
        tp: arch::MemoryType::UART,
    },  
    arch::MemoryRegion {
        start: 0x40000000,
        size: 0x40000000,
        tp: arch::MemoryType::MEM,
    },  
];

const fn mem_region(tp: arch::MemoryType) -> &'static arch::MemoryRegion { 
    let mut i = 0;
    
    loop {
        if MemoryLayout[i].tp == tp {
            return &MemoryLayout[i];
        }

        i += 1;
        if i >= MemoryLayout.len() {
            break;
        }
    }

    panic!()
}

pub const fn uart_base() -> *mut u8 {
    mem_region(arch::MemoryType::UART).start as *mut u8
}

pub const fn ram_base() -> *mut u8 {
    mem_region(arch::MemoryType::MEM).start as *mut u8
}

pub const fn ram_size() -> usize {
    mem_region(arch::MemoryType::MEM).size
}
