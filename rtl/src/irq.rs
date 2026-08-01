#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum IrqTrigger {
    Level = 0,
    Edge = 1,
}

impl TryFrom<usize> for IrqTrigger {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::Level as usize => Ok(Self::Level),
            _ if value == Self::Edge as usize => Ok(Self::Edge),
            _ => Err(()),
        }
    }
}
