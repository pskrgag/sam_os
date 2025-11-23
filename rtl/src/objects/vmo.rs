#[repr(usize)]
pub enum VmoFlags {
    Backed = 0,
    Zeroed = 1,
}

impl TryFrom<usize> for VmoFlags {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::Backed as usize => Ok(Self::Backed),
            _ if value == Self::Zeroed as usize => Ok(Self::Zeroed),
            _ => Err(()),
        }
    }
}
