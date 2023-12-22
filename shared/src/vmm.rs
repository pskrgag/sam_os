use bitflags::bitflags;

bitflags! {
pub struct MappingType: usize {
    const KernelData = 1;
    const KernelText = 2;
    const KernelDataRo = 3;
    const KernelRWX = 4;
    const KernelDevice = 5;
    const KernelNothing = 6;

    const UserData = 7;
    const UserText = 8;
    const UserDataRo = 9;
    const None = 10;
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
