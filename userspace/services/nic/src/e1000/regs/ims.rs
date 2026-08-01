use zerocopy::{FromBytes, Immutable, IntoBytes};

/// E1000 interrupt set 
#[repr(transparent)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone)]
pub struct Ims(u32);

impl Ims {
    // Receiver Timer Interrupt
    pub const INT_RXT0: u32 = 1 << 7;

    pub fn set(self, mask: u32, enabled: bool) -> Self {
        if enabled {
            Self(self.0 | mask)
        } else {
            Self(self.0 & !mask)
        }
    }
}
