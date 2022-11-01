use bitflags::bitflags;

bitflags! {
    struct AccesFlags: u8 {
        const AP_UN_KRW = 0b00;
        const AP_URW_KRW = 0b01;
        const AP_UN_KRO = 0b10;
        const AP_URO_KRO = 0b11;
    }
}

const fn access_perms(perms: AccesFlags) -> u64 {
    (perms.bits() << 6) as u64
}

const fn mair_type(idx: u8) -> u64 {
    (idx << 2) as u64
}

/* Page Block */

pub const BLOCK_VALID: u64 = 0b01;
pub const BLOCK_PXN: u64 = 1 << 53;
pub const BLOCK_NON_GLOBAL: u64 = 1 << 11;
pub const BLOCK_ACCESS_FLAG: u64 = 1 << 10;

/* Based on MAIR settings from mm::init() */
pub const BLOCK_NORMAL_MEM: u64 = mair_type(1);
pub const BLOCK_DEVICE_MEM: u64 = mair_type(0);

pub const BLOCK_KERNEL_RWX: u64 = access_perms(AccesFlags::AP_UN_KRW);
pub const BLOCK_KERNEL_RW: u64 = access_perms(AccesFlags::AP_UN_KRW) | BLOCK_PXN;
pub const BLOCK_KERNEL_RO: u64 = access_perms(AccesFlags::AP_UN_KRO) | BLOCK_PXN;

/* Page Table */
pub const TABLE_VALID: u64 = 0b11;
