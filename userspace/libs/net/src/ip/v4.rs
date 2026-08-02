use core::fmt::{self, Debug, Formatter};

/// IPv4 address
#[derive(Copy, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct IPv4([u8; 4]);

/// IPv4 config
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

impl From<[u8; 4]> for IPv4 {
    fn from(value: [u8; 4]) -> Self {
        Self(value)
    }
}

impl Debug for IPv4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}
