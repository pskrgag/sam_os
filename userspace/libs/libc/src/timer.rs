use crate::handle::Handle;
use crate::syscalls::Syscall;
use core::time::Duration;
use rtl::error::ErrorType;
use rtl::signal::Signal;
use crate::factory::factory;

pub struct Timer {
    h: Handle,
}

impl Timer {
    pub fn create() -> Result<Self, ErrorType> {
        factory().create_timer()
    }

    pub unsafe fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn wait(&self) -> Result<(), ErrorType> {
        Syscall::object_wait(&self.h, Signal::TimerReady.into())
    }

    pub fn arm(&self, deadline: Duration) -> Result<(), ErrorType> {
        Syscall::timer_arm(&self.h, deadline)
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }

    pub fn try_clone(&self) -> Result<Self, ErrorType> {
        let handle = self.h.clone_handle()?;

        Ok(unsafe { Self::new(handle) })
    }
}
