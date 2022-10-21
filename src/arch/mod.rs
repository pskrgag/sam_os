#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[derive(Copy, Clone)]
pub enum MemoryType {
    MEM,
    DEVICE,
}

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
