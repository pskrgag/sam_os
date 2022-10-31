#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

// FIXME one day...
#[path = "aarch64/qemu/config.rs"]
mod config;

#[derive(Copy, Clone)]
pub enum MemoryType {
    MEM,
    DEVICE,
}

use crate::mm::types::PhysAddr;

impl const PartialEq for MemoryType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&MemoryType::MEM, &MemoryType::MEM) |
            (&MemoryType::DEVICE, &MemoryType::DEVICE) => true,
            (x, y) => *x as u8 == *y as u8,
        } 
    }
}

pub struct MemoryRegion {
    pub start: usize,
    pub size: usize,
    pub tp: MemoryType,
}

const fn mem_region(tp: MemoryType) -> &'static MemoryRegion { 
    let mut i = 0;
    
    loop {
        if config::MEMORY_LAYOUT[i].tp == tp {
            return &config::MEMORY_LAYOUT[i];
        }

        i += 1;
        if i >= config::MEMORY_LAYOUT.len() {
            break;
        }
    }

    panic!()
}

pub const fn uart_base() -> *mut u8 {
    0x09000000 as *mut u8
}

pub const fn ram_base() -> PhysAddr {
    PhysAddr::new(mem_region(MemoryType::MEM).start as u64) 
}

pub const fn ram_size() -> usize {
    mem_region(MemoryType::MEM).size
}
