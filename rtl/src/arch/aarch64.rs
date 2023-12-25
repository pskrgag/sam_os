pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

#[cfg(feature = "kernel")]
pub const PHYS_OFFSET: usize = 0xffffffff80000000;
