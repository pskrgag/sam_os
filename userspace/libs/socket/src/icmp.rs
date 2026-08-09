use crate::bindings_NetStack::Proto;
use crate::socket::{Socket, SocketProtocol};
use heapless::Vec;
use net::ipv4::IPv4;
pub use netstack_types::icmp::EchoRequest;
use rtl::error::ErrorType;
use zerocopy::IntoBytes;

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
}
