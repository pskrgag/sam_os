use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[repr(C)]
#[derive(Debug, FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned)]
pub struct EchoRequest {
    identifier: [u8; 2],
    sequence: [u8; 2],
}

impl EchoRequest {
    pub fn new(identifier: u16, sequence: u16) -> Self {
        Self {
            identifier: identifier.to_ne_bytes(),
            sequence: sequence.to_ne_bytes(),
        }
    }

    pub fn identifier(&self) -> u16 {
        u16::from_ne_bytes(self.identifier)
    }

    pub fn sequence(&self) -> u16 {
        u16::from_ne_bytes(self.sequence)
    }
}
