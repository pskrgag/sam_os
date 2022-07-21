use crate::arch;

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
