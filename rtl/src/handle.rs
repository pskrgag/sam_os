pub type HandleBase = usize;

// To make it positive when converted to isize
pub const HANDLE_INVALID: HandleBase = HandleBase::MAX & ((1 << 63) - 1);

#[cfg(feature = "user")]
pub type Handle = HandleBase;
