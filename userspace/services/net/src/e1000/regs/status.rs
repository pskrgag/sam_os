use zerocopy::{FromBytes, Immutable, IntoBytes};

/// E1000 status register
#[repr(transparent)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone)]
pub struct Status(u32);

impl Status {
    // Link-up bit
    pub const LU: u32 = 1 << 1;

    pub fn is_set(self, mask: u32) -> bool {
        self.0 & mask == mask
    }
}
