use zerocopy::{FromBytes, Immutable, IntoBytes};

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

const _: () = {
    assert!(core::mem::size_of::<Rdesc>() == 16);
};
