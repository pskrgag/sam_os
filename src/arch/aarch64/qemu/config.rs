use crate::arch;

pub const MEMORY_LAYOUT: [arch::MemoryRegion; 2] = [
    arch::MemoryRegion {
        start: 0x08000000,
        size: 0x02000000,
        tp: arch::MemoryType::DEVICE,
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
        if MEMORY_LAYOUT[i].tp == tp {
            return &MEMORY_LAYOUT[i];
        }

        i += 1;
        if i >= MEMORY_LAYOUT.len() {
            break;
        }
    }

    panic!()
}

pub const fn uart_base() -> *mut u8 {
    0x09000000 as *mut u8
}

pub const fn ram_base() -> *mut u8 {
    mem_region(arch::MemoryType::MEM).start as *mut u8
}

pub const fn ram_size() -> usize {
    mem_region(arch::MemoryType::MEM).size
}
