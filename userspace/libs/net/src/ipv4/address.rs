use core::fmt::{self, Debug, Formatter};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(
    Copy,
    Clone,
    Ord,
    Eq,
    PartialEq,
    PartialOrd,
    FromBytes,
    Immutable,
    KnownLayout,
    Unaligned,
    IntoBytes,
)]
#[repr(C)]
pub struct IPv4([u8; 4]);

pub struct Ipv4Config {
    pub address: IPv4,
    pub prefix_len: u8,
    pub gateway: Option<IPv4>,
}

impl IPv4 {
    pub fn new(first: u8, second: u8, third: u8, forth: u8) -> Self {
        Self([first, second, third, forth])
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn is_anycast(&self) -> bool {
        self.0 == [0xff, 0xff, 0xff, 0xff]
    }
}

impl TryFrom<&str> for IPv4 {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut raw = [0; 4];

        for (i, val) in value.split('.').enumerate() {
            if i >= 4 {
                return Err(());
            }

            raw[i] = val.parse::<u8>().map_err(|_| ())?;
        }

        Ok(Self(raw))
    }
}

impl From<[u8; 4]> for IPv4 {
    fn from(value: [u8; 4]) -> Self {
        Self(value)
    }
}

impl From<u32> for IPv4 {
    fn from(value: u32) -> Self {
        Self(value.to_ne_bytes())
    }
}

impl Into<[u8; 4]> for IPv4 {
    fn into(self) -> [u8; 4] {
        self.0
    }
}

impl Debug for IPv4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}
