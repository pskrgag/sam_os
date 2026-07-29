use zerocopy::{FromBytes, Immutable, IntoBytes};

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

const _: () = {
    assert!(core::mem::size_of::<Tdesc>() == 16);
};
