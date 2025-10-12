use super::syscalls::Syscall;
use rtl::{error::ErrorType, handle};

/// Owning RAII wrapper around handle
#[repr(transparent)]
#[derive(Debug)]
pub struct Handle(handle::Handle);

impl Handle {
    pub fn new(h: handle::Handle) -> Self {
        Self(h)
    }

    /// # SAFETY
    /// don't use it, unless you know what you are doing
    pub unsafe fn as_raw(&self) -> handle::Handle {
        self.0
    }

    pub fn clone_handle(&self) -> Result<Self, ErrorType> {
        Syscall::clone_handle(self)
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // TODO: panic here
        let _ = Syscall::close_handle(self.0);
    }
}
