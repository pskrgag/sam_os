use crate::bindings_NetStack::{Address, Proto, Socket as BindingSocket};
use core::marker::PhantomData;
use heapless::Vec;
use libc::handle::Handle;
use net::{header::Header, ipv4::IPv4};
use rokio::port::Port;
use rtl::error::ErrorType;
use zerocopy::IntoBytes;

pub trait SocketProtocol {
    type Header: Header<Error = ErrorType> + 'static;

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

    pub async fn send_to(
        &self,
        address: IPv4,
        header: &P::Header,
        payload: &[u8],
    ) -> Result<usize, ErrorType> {
        let mut data: Vec<u8, 4096> = Vec::new();
        data.extend_from_slice(header.as_bytes())
            .map_err(|_| ErrorType::BufferTooBig)?;
        data.extend_from_slice(payload)
            .map_err(|_| ErrorType::BufferTooBig)?;

        let address = Address {
            ipv4: u32::from_be_bytes(address.as_slice().try_into().unwrap()),
        };

        Ok(self.socket.SendTo(address, data).await?.sent)
    }

    pub async fn receive<'a>(
        &self,
        data: &'a mut [u8],
    ) -> Result<(&'a P::Header, &'a [u8]), ErrorType> {
        if data.len() > 4096 {
            return Err(ErrorType::BufferTooBig);
        }

        let response = self.socket.Receive(data.len()).await?;
        let read = response.data.len();
        data[..read].copy_from_slice(response.data.as_slice());
        let packet = &data[..read];
        let header = P::Header::parse(packet)?;
        let payload = &packet[P::Header::fixed_len()..];

        Ok((header, payload))
    }
}
