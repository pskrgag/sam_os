use crate::arch;
use shared::vmm::types::PhysAddr;

pub const MEMORY_LAYOUT: [arch::MemoryRegion; 2] = [
    arch::MemoryRegion {
        start: 0x08000000,
        size: 0x02000000,
        tp: arch::MemoryType::DEVICE,
    },
    arch::MemoryRegion {
        start: 0x40000000,
        size: 0x0200000,
        tp: arch::MemoryType::MEM,
    },
];

pub const fn uart_base() -> *mut u8 {
    0x09000000 as *mut u8
}

pub const fn ram_base() -> *mut u8 {
    0x40000000 as *mut u8
}

pub const fn ram_size() -> usize {
    0x02000000 / 4
}

pub const fn gic_dist() -> (PhysAddr, usize) {
    (PhysAddr::new(0x08000000), 0x1000)
}

pub const fn gic_cpu() -> (PhysAddr, usize) {
    (PhysAddr::new(0x08010000), 0x1000)
}
