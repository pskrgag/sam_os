use crate::bindings_NetStack::{Address, Proto, Socket as BindingSocket};
use core::marker::PhantomData;
use heapless::Vec;
use libc::handle::Handle;
use net::ipv4::IPv4;
use rokio::port::Port;
use rtl::error::ErrorType;

pub trait SocketProtocol {
    const PROTO: Proto;
}

pub struct Socket<P: SocketProtocol> {
    socket: BindingSocket,
    protocol: PhantomData<P>,
}

impl<P: SocketProtocol> Socket<P> {
    /// # Safety
    /// `handle` must refer to a Socket service port.
    pub(crate) unsafe fn new(handle: Handle) -> Self {
        Self {
            socket: BindingSocket::new(unsafe { Port::new(handle) }),
            protocol: PhantomData,
        }
    }

    pub(crate) async fn send_to_raw(
        &self,
        address: IPv4,
        data: Vec<u8, 4096>,
    ) -> Result<usize, ErrorType> {
        let address = Address {
            ipv4: u32::from_ne_bytes(address.as_slice().try_into().unwrap()),
        };

        Ok(self.socket.SendTo(address, data).await?.sent)
    }
}
