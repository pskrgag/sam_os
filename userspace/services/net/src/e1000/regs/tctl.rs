use zerocopy::{FromBytes, Immutable, IntoBytes};

/// E1000 control register
#[repr(transparent)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone)]
pub struct Tctl(u32);

impl Tctl {
    pub const ENABLE: u32 = 1 << 1;
    pub const PSP: u32 = 1 << 3;

    pub fn collision_threshold(val: u32) -> u32 {
        val << 4
    }

    pub fn collision_distance(val: u32) -> u32 {
        val << 12
    }

    pub fn set(self, mask: u32, enabled: bool) -> Self {
        if enabled {
            Self(self.0 | mask)
        } else {
            Self(self.0 & !mask)
        }
    }
}
