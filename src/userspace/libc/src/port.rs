use crate::factory::factory;
use crate::syscalls::Syscall;
use rtl::error::*;
use rtl::handle::Handle;
use rtl::ipc::message::IpcMessage;
use rtl::misc::ref_to_usize;
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

    pub fn receive(&self, msg: &mut IpcMessage) -> Result<(), ErrorType> {
        Syscall::invoke(self.h, PortInvoke::RECEIVE.bits(), &[ref_to_usize(msg)]).map(|_| ())
    }

    pub fn send_and_wait(
        &self,
        reply_port: Port,
        msg: &IpcMessage,
        reply: &mut IpcMessage,
    ) -> Result<(), ErrorType> {
        Syscall::invoke(
            self.h,
            PortInvoke::SEND_AND_WAIT.bits(),
            &[reply_port.handle(), ref_to_usize(msg), ref_to_usize(reply)],
        )
        .map(|_| ())
    }

    pub fn call(&self, send: &mut IpcMessage, reply: &mut IpcMessage) -> Result<(), ErrorType> {
        let p = Port::create().ok_or(ErrorType::NO_OPERATION)?;

        send.set_reply_port(p.handle());
        Syscall::invoke(self.h, PortInvoke::CALL.bits(), &[ref_to_usize(send), ref_to_usize(reply)]).map(|_| ())
    }

    pub fn handle(&self) -> Handle {
        self.h
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        Syscall::invoke(self.h, rtl::handle::HANDLE_CLOSE, &[]).unwrap();
    }
}
