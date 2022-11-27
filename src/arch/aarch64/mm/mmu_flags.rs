use bitflags::bitflags;

bitflags! {
    struct AccesFlags: u8 {
        const AP_UN_KRW = 0b00;
        const AP_URW_KRW = 0b01;
        const AP_UN_KRO = 0b10;
        const AP_URO_KRO = 0b11;
    }
}

const fn access_perms(perms: AccesFlags) -> usize {
    (perms.bits() << 6) as usize
}

const fn mair_type(idx: u8) -> usize {
    (idx << 2) as usize
}

/* Page Block */

pub const BLOCK_VALID: usize = 0b01;
pub const BLOCK_PXN: usize = 1 << 53;
pub const BLOCK_NON_GLOBAL: usize = 1 << 11;
pub const BLOCK_ACCESS_FLAG: usize = 1 << 10;

/* Based on MAIR settings from mm::init() */
pub const BLOCK_NORMAL_MEM: usize = mair_type(0);
pub const BLOCK_DEVICE_MEM: usize = mair_type(1);

pub const BLOCK_KERNEL_RWX: usize = access_perms(AccesFlags::AP_UN_KRW);
pub const BLOCK_KERNEL_RW: usize = access_perms(AccesFlags::AP_UN_KRW) | BLOCK_PXN;
pub const BLOCK_KERNEL_RO: usize = access_perms(AccesFlags::AP_UN_KRO) | BLOCK_PXN | (1 << 54) | (1 << 51);

/* Page Table */
pub const TABLE_VALID: usize = 0b11;

pub const PAGE_ENTRY_FLAGS_MASK: usize = 0xFFF0_0000_0000_0FFF;
