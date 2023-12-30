use bitflags::bitflags;

pub mod alloc;
pub mod slab;
pub mod types;

bitflags! {
pub struct MappingType: usize {
    const KERNEL_DATA = 1;
    const KERNEL_TEXT = 2;
    const KERNEL_DATA_RO = 4;
    const KERNEL_RWX = 8;
    const KERNEL_DEVICE = 16;

    const USER_DATA = 32;
    const USER_TEXT = 64;
    const USER_DATA_RO = 128;
    const NONE = 256;
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
