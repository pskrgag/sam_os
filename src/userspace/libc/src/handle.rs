use super::syscalls::Syscall;
use rtl::handle;

/// Owning RAII wrapper around handle
#[repr(transparent)]
pub struct Handle(handle::Handle);

impl Handle {
    pub fn new(h: handle::Handle) -> Self {
        Self(h)
    }

    pub(crate) unsafe fn as_raw(&self) -> handle::Handle {
        self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        Syscall::close_handle(self.0).expect("Failed to close handle");
    }
}
