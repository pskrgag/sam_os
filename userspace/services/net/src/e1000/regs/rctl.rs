use zerocopy::{FromBytes, Immutable, IntoBytes};

/// E1000 control register
#[repr(transparent)]
#[derive(Immutable, FromBytes, IntoBytes, Copy, Clone)]
pub struct Rctl(u32);

impl Rctl {
    pub const ENABLE: u32 = 1 << 1;
    pub const BAM: u32 = 1 << 15;

    pub fn bsize(size: usize) -> u32 {
        match size {
            256 => 0b11 << 16,
            512 => 0b10 << 16,
            1024 => 0b01 << 16,
            2048 => 0b00 << 16,
            4096 => (0b11 << 16) | (1 << 25),
            8192 => (0b10 << 16) | (1 << 25),
            16384 => (0b01 << 16) | (1 << 25),
            _ => panic!("Invalid argument"),
        }
    }

    pub fn set(self, mask: u32, enabled: bool) -> Self {
        if enabled {
            Self(self.0 | mask)
        } else {
            Self(self.0 & !mask)
        }
    }
}
