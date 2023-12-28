pub type HandleBase = usize;

// const HANDLE_INVALID: HandleBase = HandleBase::MAX;

#[cfg(feature = "user")]
pub type Handle = HandleBase;
