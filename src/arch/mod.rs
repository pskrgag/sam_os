pub mod qemu;

pub enum MemoryType {
    MEM,
    UART,
}

pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub tp: MemoryType,
}
