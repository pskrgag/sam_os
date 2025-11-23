pub mod alloc;
pub mod slab;

#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum MappingType {
    None = 0,
    Data = 1 << 0,
    Text = 1 << 1,
    RoData = 1 << 2,
    Rwx = 1 << 3,

    // TODO: add more granular params
    Device = 1 << 4,
}

impl TryFrom<usize> for MappingType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::None as usize => Ok(Self::None),
            _ if value == Self::Data as usize => Ok(Self::Data),
            _ if value == Self::Text as usize => Ok(Self::Text),
            _ if value == Self::RoData as usize => Ok(Self::RoData),
            _ if value == Self::Rwx as usize => Ok(Self::Rwx),
            _ if value == Self::Device as usize => Ok(Self::Device),
            _ => Err(()),
        }
    }
}
