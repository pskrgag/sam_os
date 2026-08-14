use crate::bindings_NetStack::Proto;
use crate::socket::{Socket, SocketProtocol};
use heapless::Vec;
use net::ipv4::IPv4;
pub use netstack_types::icmp::EchoRequest;
use rtl::error::ErrorType;
use zerocopy::{FromBytes, IntoBytes};

pub struct Icmp;

impl SocketProtocol for Icmp {
    const PROTO: Proto = Proto::ICMP;
}

impl Socket<Icmp> {
    pub async fn send_to(
        &self,
        address: IPv4,
        request: &EchoRequest,
        payload: &[u8],
    ) -> Result<usize, ErrorType> {
        let mut data: Vec<u8, 4096> = Vec::new();

        data.extend_from_slice(request.as_bytes())
            .map_err(|_| ErrorType::BufferTooBig)?;
        data.extend_from_slice(payload)
            .map_err(|_| ErrorType::BufferTooBig)?;

        self.send_to_raw(address, data).await
    }

    pub async fn recv_from(
        &self,
        request: &mut EchoRequest,
        payload: &mut [u8],
    ) -> Result<(usize, IPv4), ErrorType> {
        let mut data: Vec<u8, 4096> = Vec::new();
        let header_size = core::mem::size_of::<EchoRequest>();
        let (size, address) = self
            .recv_from_raw(&mut data, header_size + payload.len())
            .await?;

        let data = data.get(..size).ok_or(ErrorType::BufferTooSmall)?;
        let (reply, reply_payload) =
            EchoRequest::ref_from_prefix(data).map_err(|_| ErrorType::BufferTooSmall)?;
        let payload_size = reply_payload.len().min(payload.len());

        *request = EchoRequest::new(reply.identifier(), reply.sequence());
        payload[..payload_size].copy_from_slice(&reply_payload[..payload_size]);

        Ok((payload_size, address))
    }
}
