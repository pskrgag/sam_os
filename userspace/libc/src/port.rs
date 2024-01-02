use crate::syscalls::Syscall;
use crate::factory::factory;
use rtl::handle::Handle;
use rtl::objects::port::PortInvoke;

pub struct Port {
    h: Handle,
}

impl Port {
    pub fn new(h: Handle) -> Self {
        Self { h }
    }
    
    pub fn create() -> Option<Self> {
        factory().create_port()
    }

    pub fn receive(&self) {
        Syscall::invoke(self.h, PortInvoke::RECEIVE.bits(), &[]).unwrap();
    }

    pub fn send(&self) {
        Syscall::invoke(self.h, PortInvoke::CALL.bits(), &[]).unwrap();
    }

    pub fn handle(&self) -> Handle {
        self.h
    }
}
