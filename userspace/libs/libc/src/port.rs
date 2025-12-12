use super::handle::Handle;
use crate::factory::factory;
use crate::syscalls::Syscall;
use rtl::error::*;
use rtl::ipc::message::IpcMessage;

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

    pub fn send_and_wait(
        &self,
        reply_port: Handle,
        reply: &mut IpcMessage,
    ) -> Result<usize, ErrorType> {
        Syscall::port_send_wait(&self.h, reply_port, reply)
    }

    pub fn send(
        &self,
        reply_port: Handle,
        reply: &mut IpcMessage,
    ) -> Result<(), ErrorType> {
        Syscall::port_send(&self.h, reply_port, reply)
    }

    pub fn call(&self, msg: &mut IpcMessage) -> Result<usize, ErrorType> {
        let p = Port::create().ok_or(ErrorType::NoOperation)?;

        msg.set_reply_port(unsafe { p.h.as_raw() });
        Syscall::port_call(&self.h, msg)
    }

    pub fn receive(&self, msg: &mut IpcMessage) -> Result<usize, ErrorType> {
        Syscall::port_receive(&self.h, msg)
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }
}
