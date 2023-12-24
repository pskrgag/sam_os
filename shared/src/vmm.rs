use bitflags::bitflags;

bitflags! {
pub struct MappingType: usize {
    const KERNEL_DATA = 1;
    const KERNEL_TEXT = 2;
    const KERNEL_DATA_RO = 3;
    const KERNEL_RWX = 4;
    const KERNEL_DEVICE = 5;

    const USER_DATA = 7;
    const USER_TEXT = 8;
    const USER_DATA_RO = 9;
    const NONE = 10;
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
