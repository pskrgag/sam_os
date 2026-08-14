use super::packet::Packet;
use crate::netstack::NetStack;
use adt::AsyncDeq;
use alloc::boxed::Box;
use alloc::sync::Arc;
use net::ipv4::{IPv4, IPv4Header};
use rtl::error::ErrorType;

pub mod server;

#[async_trait::async_trait]
pub trait SocketOps: Send + Sized {
    fn new() -> Self;

    async fn send_to(
        &self,
        sock: &Arc<Socket<Self>>,
        netstack: &Arc<NetStack>,
        address: IPv4,
        data: &[u8],
    ) -> Result<(), ErrorType>;

    async fn receive(
        &self,
        sock: &Arc<Socket<Self>>,
        netstack: &Arc<NetStack>,
        data: &mut [u8],
    ) -> Result<(), ErrorType>;
}

pub struct Socket<S: SocketOps> {
    ops: S,
    rx: AsyncDeq<Packet>,
    netstack: Arc<NetStack>,
}

impl<S: SocketOps> Socket<S> {
    pub fn new(ops: S, netstack: Arc<NetStack>) -> Arc<Self> {
        Arc::new(Self {
            netstack,
            ops,
            rx: AsyncDeq::new(),
        })
    }

    pub fn push_packet(self: &Arc<Self>, packet: Packet) {
        self.rx.push_back(packet);
    }

    pub async fn send_to(self: Arc<Self>, address: IPv4, data: &[u8]) -> Result<(), ErrorType> {
        self.ops.send_to(&self, &self.netstack, address, data).await
    }

    pub async fn recv_from(self: Arc<Self>, data: &mut [u8]) -> Result<(usize, IPv4), ErrorType> {
        let packet = self.rx.pop_front().await;
        let size = packet.payload().len().min(data.len());
        let source = packet.network_header::<IPv4Header>().source();

        data.copy_from_slice(&packet.payload()[..size]);
        Ok((size, source))
    }
}
