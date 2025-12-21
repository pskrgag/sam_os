#![no_std]

use alloc::{boxed::Box, vec::Vec};
use core::iter::zip;
use libc::port::Port;
use libc::syscalls::Syscall;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;
use rtl::signal::{Signal, WaitEntry};

extern crate alloc;

pub trait Endpoint: Send {
    /// Returns underlying port
    fn port(&self) -> &Port;

    /// Handles one message
    fn handle_one(&mut self) -> Result<(), ErrorType>;
}

pub struct EndpointsDispatcher {
    endpoints: Spinlock<Vec<Box<dyn Endpoint>>>,
}

impl EndpointsDispatcher {
    /// Contracts empty dispatcher pool
    pub const fn new() -> Self {
        Self {
            endpoints: Spinlock::new(Vec::new()),
        }
    }

    /// Adds new dispatcher
    pub fn add(&self, new: Box<dyn Endpoint>) {
        self.endpoints.lock().push(new)
    }

    /// Dispatches one message
    pub fn dispatch_one(&self) -> Result<(), ErrorType> {
        let mut wait_entries = self
            .endpoints
            .lock()
            .iter()
            .map(|x| WaitEntry {
                handle: unsafe { x.port().handle().as_raw() },
                waitfor: Signal::MessageReady.into(),
                pendind: Signal::None.into(),
            })
            .collect::<Vec<_>>();

        Syscall::object_wait_many(&mut wait_entries)?;

        // Swap with empty one to allow adding new endpoints in handlers
        let mut new = Vec::with_capacity(self.endpoints.lock().len());
        core::mem::swap(&mut *self.endpoints.lock(), &mut new);

        for mut entries in
            zip(new, wait_entries).filter(|(_, we)| *(we.pendind & Signal::MessageReady) != 0)
        {
            entries.0.handle_one().unwrap();
            self.endpoints.lock().push(entries.0);
        }

        Ok(())
    }

    /// Dispatches until first error
    pub fn dispatch(&self) -> Result<(), ErrorType> {
        loop {
            self.dispatch_one()?;
        }
    }
}

impl Default for EndpointsDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
