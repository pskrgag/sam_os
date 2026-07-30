use zerocopy::{FromBytes, Immutable, IntoBytes};

const RX_STATUS_DD: u8 = 1 << 0;
const RX_STATUS_EOP: u8 = 1 << 1;

/// E1000 receive descriptor
#[repr(packed)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone, Default)]
pub struct Rdesc {
    pub buffer: u64,
    pub length: u16,
    pub crc: u16,
    pub status: u8,
    pub errors: u8,
    pub special: u16,
}

impl Rdesc {
    pub fn is_ready(&self) -> bool {
        self.status & RX_STATUS_DD != 0
    }

    pub fn ack(&mut self) {
        self.status = 0;
    }
}

const _: () = {
    assert!(core::mem::size_of::<Rdesc>() == 16);
};
