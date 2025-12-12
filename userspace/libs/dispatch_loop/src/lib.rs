#![no_std]

use alloc::{boxed::Box, vec::Vec};
use libc::port::Port;
use rtl::error::ErrorType;

extern crate alloc;

pub trait Endpoint: Send {
    /// Returns underlying port
    fn port(&self) -> &Port;

    /// Handles one message
    fn handle_one(&mut self) -> Result<(), ErrorType>;
}

pub struct EndpointsDispatcher {
    endpoints: Vec<Box<dyn Endpoint>>,
}

impl EndpointsDispatcher {
    pub const fn new() -> Self {
        Self {
            endpoints: Vec::new(),
        }
    }

    pub fn add(&mut self, new: Box<dyn Endpoint>) {
        self.endpoints.push(new)
    }
}

impl Default for EndpointsDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
