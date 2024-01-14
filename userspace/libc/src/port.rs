use crate::factory::factory;
use crate::syscalls::Syscall;
use rtl::handle::Handle;
use rtl::ipc::*;
use rtl::misc::ref_to_usize;
use rtl::objects::port::PortInvoke;
use rtl::error::*;
use rtl::handle::*;

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

    pub fn receive_data<T: Copy>(&self, data: &mut T) -> Result<Option<Port>, ErrorType> {
        let mut msg = IpcMessage::default();
        msg.set_out_data(data);

        match Syscall::invoke(self.h, PortInvoke::RECEIVE.bits(), &[ref_to_usize(&msg)]) {
            Ok(h) => {
                if h != HANDLE_INVALID {
                    Ok(Some(Port::new(h)))
                } else {
                    Ok(None)
                }
            },
            Err(e) => Err(e)
        }
    }

    pub fn send_data<T: Copy>(&self, reply_port: Port, data: &T) {
        let mut msg = IpcMessage::default();
        msg.add_data(data);

        Syscall::invoke(self.h, PortInvoke::SEND.bits(), &[reply_port.handle(), ref_to_usize(&msg)]).unwrap();
    }


    // Copy is kinda POD-like from C
    pub fn call<T: Copy, U: Copy>(
        &self,
        in_data: &T,
        out_data: Option<&mut U>,
        reply_port: Option<&Port>,
    ) {
        let mut msg = IpcMessage::default();

        msg.add_data(in_data);

        if let Some(data) = out_data {
            msg.set_out_data(data);
        }

        if let Some(r) = reply_port{
            msg.set_reply_port(r.handle());
        }

        Syscall::invoke(self.h, PortInvoke::CALL.bits(), &[ref_to_usize(&msg)]).unwrap();
    }

    pub fn handle(&self) -> Handle {
        self.h
    }
}
