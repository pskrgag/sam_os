use zerocopy::{FromBytes, Immutable, IntoBytes};

// Last packet
const CMD_EOP: u8 = 1 << 0;
// CRC
const CMD_IFCS: u8 = 1 << 1;
// Report status
const CMD_RS: u8 = 1 << 3;

// Buffer is consumed
const STATUS_DD: u8 = 1 << 0;

/// E1000 transmit descriptor
#[repr(packed)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone, Default)]
pub struct Tdesc {
    pub buffer: u64,
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
    pub status: u8,
    pub css: u8,
    pub special: u16,
}

impl Tdesc {
    pub fn is_ready(&self) -> bool {
        self.status & STATUS_DD != 0
    }

    pub fn new(buffer: u64, length: u16) -> Self {
        Tdesc {
            buffer,
            length,
            cmd: CMD_RS | CMD_IFCS | CMD_EOP,
            ..Default::default()
        }
    }
}

const _: () = {
    assert!(core::mem::size_of::<Tdesc>() == 16);
};
