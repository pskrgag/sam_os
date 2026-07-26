use zerocopy::{FromBytes, Immutable, IntoBytes};

/// E1000 control register
#[repr(transparent)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone)]
pub struct Tctl(u32);

impl Tctl {
    pub const ENABLE: u32 = 1 << 1;

    pub fn set(self, mask: u32, enabled: bool) -> Self {
        if enabled {
            Self(self.0 | mask)
        } else {
            Self(self.0 & !mask)
        }
    }
}
