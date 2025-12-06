#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum MappingType {
    None = 0,
    Data = 1,
    RoData = 2,
    Text = 3,
    Rwx = 4,

    // TODO: add more granular params
    Device = 5,
}

impl MappingType {
    pub fn is_greater(&self, other: MappingType) -> bool {
        #[repr(usize)]
        enum Flags {
            Read = 1,
            Write = 2,
            Execute = 4,
        }

        fn into(tp: MappingType) -> usize {
            match tp {
                MappingType::None => 0,
                MappingType::Data | MappingType::Device => {
                    Flags::Read as usize | Flags::Write as usize
                }
                MappingType::RoData => Flags::Read as usize,
                MappingType::Text => Flags::Read as usize | Flags::Execute as usize,
                MappingType::Rwx => {
                    Flags::Read as usize | Flags::Execute as usize | Flags::Write as usize
                }
            }
        }

        let current = into(self.clone());
        let other = into(other);

        (current ^ other) & current != 0
    }
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
