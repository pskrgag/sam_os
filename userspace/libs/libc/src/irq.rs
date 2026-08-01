use crate::handle::Handle;
use crate::syscalls::Syscall;
use rtl::error::ErrorType;

pub struct Irq {
    h: Handle,
}

impl Irq {
    pub unsafe fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn wait(&self) -> Result<(), ErrorType> {
        Syscall::wait_irq(&self.h)
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }

    pub fn try_clone(&self) -> Result<Self, ErrorType> {
        let res = self.h.clone_handle()?;

        unsafe { Ok(Self::new(res)) }
    }
}
