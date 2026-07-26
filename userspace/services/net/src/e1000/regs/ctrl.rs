use zerocopy::{Immutable, FromBytes, IntoBytes};

/// E1000 control register
#[repr(transparent)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone)]
pub struct Control(u32);

impl Control {
    pub const LINK_RESET: u32 = 1 << 3;
    pub const RESET: u32 = 1 << 26;

    pub fn set(self, mask: u32, enabled: bool) -> Self {
        if enabled {
            Self(self.0 | mask)
        } else {
            Self(self.0 & !mask)
        }
    }

    pub fn is_set(self, mask: u32) -> bool {
        self.0 & mask == mask
    }
}
