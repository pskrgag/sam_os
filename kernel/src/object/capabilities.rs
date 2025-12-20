use bitmask::bitmask;

bitmask! {
    pub mask CapabilityBits: u8 where flags Capability {
        None = 0,

        // Ports
        Send = (1 << 0),
        Receive = (1 << 1),
        Call = (1 << 2),

        // VmSpace
        MapPhys = (1 << 3),
    }
}

#[derive(Clone)]
pub struct CapabilityMask(CapabilityBits);

impl CapabilityMask {
    pub fn any() -> Self {
        Self(CapabilityBits::from(Capability::None))
    }

    pub fn from<T: Into<CapabilityBits>>(caps: T) -> Self {
        Self(caps.into())
    }

    pub fn is_set(&self, caps: CapabilityMask) -> bool {
        self.0 & caps.0 == caps.0
    }
}
