use crate::factory::factory;
use crate::syscalls::Syscall;
use rtl::error::*;
use rtl::handle::Handle;
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
        Syscall::port_send_wait(self.h, reply_port, reply)
    }

    pub fn call(&self, msg: &mut IpcMessage) -> Result<(), ErrorType> {
        let p = Port::create().ok_or(ErrorType::NO_OPERATION)?;

        msg.set_reply_port(p.handle());
        Syscall::port_call(self.h, msg)
    }

    pub fn receive(&self, msg: &mut IpcMessage) -> Result<usize, ErrorType> {
        Syscall::port_receive(self.h, msg)
    }

    pub fn handle(&self) -> Handle {
        self.h
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        Syscall::close_handle(self.h).unwrap();
    }
}
