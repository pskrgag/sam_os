pub struct CapabilityMask(usize);

impl CapabilityMask {
    pub const fn invalid() -> Self {
        Self(0)
    }
}
