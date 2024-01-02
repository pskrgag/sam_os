pub type HandleBase = usize;

pub const HANDLE_INVALID: HandleBase = HandleBase::MAX;

#[cfg(feature = "user")]
pub type Handle = HandleBase;
