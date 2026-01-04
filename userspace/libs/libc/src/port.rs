use super::handle::Handle;
use crate::factory::factory;
use crate::syscalls::Syscall;
use rtl::error::ErrorType;
use rtl::ipc::message::IpcMessage;
use rtl::signal::Signal;

pub struct Port {
    h: Handle,
}

impl Port {
    pub unsafe fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn create() -> Result<Self, ErrorType> {
        factory().create_port()
    }

    pub fn reply_and_wait(
        &self,
        reply_port: Handle,
        reply: &mut IpcMessage,
    ) -> Result<usize, ErrorType> {
        Syscall::port_send_wait(&self.h, reply_port, reply)
    }

    pub fn reply(&self, reply_port: Handle, reply: &IpcMessage) -> Result<(), ErrorType> {
        Syscall::port_reply(&self.h, reply_port, reply)
    }

    pub fn call(&self, msg: &mut IpcMessage) -> Result<usize, ErrorType> {
        let p = Port::create()?;

        msg.set_reply_port(unsafe { p.h.as_raw() });
        Syscall::port_call(&self.h, msg)
    }

    pub fn send(&self, msg: &mut IpcMessage) -> Result<Port, ErrorType> {
        let p = Port::create()?;

        msg.set_reply_port(unsafe { p.h.as_raw() });
        Syscall::port_send(&self.h, msg).map(|_| p)
    }

    pub fn receive(&self, msg: &mut IpcMessage) -> Result<usize, ErrorType> {
        Syscall::port_receive(&self.h, msg)
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }

    pub fn wait_message(&self) {
        Syscall::object_wait(&self.h, Signal::MessageReady.into()).unwrap()
    }
}
