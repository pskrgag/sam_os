use super::packet::Packet;
use crate::netstack::NetStack;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use net::ipv4::IPv4;
use rtl::error::ErrorType;

pub mod server;

#[async_trait::async_trait]
pub trait SocketOps: Send + Sized {
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
    rx: VecDeque<Packet>,
    netstack: Arc<NetStack>,
}

impl<S: SocketOps> Socket<S> {
    pub fn new(ops: S, netstack: Arc<NetStack>) -> Arc<Self> {
        Arc::new(Self {
            netstack,
            ops,
            rx: VecDeque::new(),
        })
    }
}
