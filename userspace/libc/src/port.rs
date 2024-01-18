use crate::factory::factory;
use crate::syscalls::Syscall;
use rtl::error::*;
use rtl::handle::Handle;
use rtl::handle::*;
use rtl::ipc::*;
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

    pub fn receive_data(&self, data: &mut [u8]) -> Result<Option<Port>, ErrorType> {
        let mut msg = IpcMessage::default();
        msg.set_out_data_raw(data);

        match Syscall::invoke(self.h, PortInvoke::RECEIVE.bits(), &[ref_to_usize(&msg)]) {
            Ok(h) => {
                if h != HANDLE_INVALID {
                    Ok(Some(Port::new(h)))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn send_data(&self, reply_port: Port, data: &[u8]) {
        let mut msg = IpcMessage::default();
        msg.add_data_raw(data);

        Syscall::invoke(
            self.h,
            PortInvoke::SEND.bits(),
            &[reply_port.handle(), ref_to_usize(&msg)],
        )
        .unwrap();
    }

    pub fn call(
        &self,
        in_data: &[u8],
        out_data: Option<&mut [u8]>,
    ) -> Result<(), ErrorType> {
        let mut msg = IpcMessage::default();

        // Make lifetime till end of the function.
        let p;
        msg.add_data_raw(in_data);

        if let Some(data) = out_data {
            p = Port::create().ok_or(ErrorType::NO_OPERATION)?;
            msg.set_out_data_raw(data);

            msg.set_reply_port(p.handle());
        }

        Syscall::invoke(self.h, PortInvoke::CALL.bits(), &[ref_to_usize(&msg)]).map(|x| ())
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
