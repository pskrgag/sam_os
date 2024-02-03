#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[derive(Copy, Clone)]
pub enum MemoryType {
    MEM,
    DEVICE,
}

#[derive(Clone)]
pub struct MemoryRegion {
    pub start: usize,
    pub size: usize,
    pub tp: MemoryType,
}
