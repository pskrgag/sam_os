pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

#[cfg(feature = "kernel")]
pub const PHYS_OFFSET: usize = 0xFFFF700000000000;

pub const TCR_SZ_SHIFT: usize = 39;

pub const USER_AS_END: usize = (1 << TCR_SZ_SHIFT) - 1;
