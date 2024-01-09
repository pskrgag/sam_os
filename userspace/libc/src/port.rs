use crate::factory::factory;
use crate::syscalls::Syscall;
use rtl::handle::Handle;
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

    pub fn receive_data<T: AsRef<[u8]>>(&self, data: &mut T) {
        let mut msg = IpcMessage::default();
        msg.add_data(&data);

        Syscall::invoke(self.h, PortInvoke::RECEIVE.bits(), &[ref_to_usize(&msg)]).unwrap();
    }

    pub fn send_data<T: AsRef<[u8]>>(&self, data: T) {
        let mut msg = IpcMessage::default();
        msg.add_data(&data);

        Syscall::invoke(self.h, PortInvoke::SEND.bits(), &[ref_to_usize(&msg)]).unwrap();
    }

    pub fn handle(&self) -> Handle {
        self.h
    }
}
