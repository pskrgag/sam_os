use rtl::handle::Handle;
use crate::syscalls::Syscall;

pub struct VmObject {
    h: Handle,
}

impl VmObject {
    pub fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn handle(&self) -> Handle {
        self.h
    }
}

impl Drop for VmObject {
    fn drop(&mut self) {
        Syscall::close_handle(self.h).unwrap();
    }
}
