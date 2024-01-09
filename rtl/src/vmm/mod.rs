use bitflags::bitflags;

pub mod alloc;
pub mod slab;
pub mod types;

bitflags! {
pub struct MappingType: usize {
    const KERNEL_DATA = 1 << 0;
    const KERNEL_TEXT = 1 << 1;
    const KERNEL_DATA_RO = 1 << 2;
    const KERNEL_RWX = 1 << 3;
    const KERNEL_DEVICE = 1 << 4;

    const USER_DATA = 1 << 5;
    const USER_TEXT = 1 << 6;
    const USER_DATA_RO = 1 << 7;
    const USER_DEVICE = 1 << 8;
    const NONE = 1 << 9;
}
}

impl From<usize> for MappingType {
    fn from(value: usize) -> Self {
        Self::from_bits(value).unwrap()
    }
}

impl From<MappingType> for usize {
    fn from(value: MappingType) -> Self {
        value.bits()
    }
}
