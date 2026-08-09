use super::inet;
use crate::netdev::{arp::ArpCache, Netdev};
use crate::packet::Packet;
use alloc::sync::Arc;
use net::ethernet::{EthHeader, FrameType, Mac};
use net::ipv4::{IPv4, IPv4Header, Protocol};
use rtl::error::ErrorType;
use spin::Mutex;

#[derive(Debug)]
pub enum PacketDecision {
    Reply(Packet),
    Drop,
}

pub struct NetStack {
    netdev: Arc<Netdev>,
    arp_cache: Mutex<ArpCache>,
}

impl NetStack {
    pub fn new(netdev: Arc<Netdev>) -> Self {
        Self {
            netdev,
            arp_cache: Mutex::new(ArpCache::default()),
        }
    }

    pub fn ip_address(&self) -> IPv4 {
        self.netdev.ip_address()
    }

    pub fn mac_address(&self) -> Mac {
        self.netdev.mac_address()
    }

    pub async fn send_packet_to(
        &self,
        address: IPv4,
        proto: Protocol,
        packet: Packet,
    ) -> Result<(), ErrorType> {
        let mut packet = packet;
        let header = IPv4Header::new(
            address,
            self.ip_address(),
            proto,
            packet.payload_len() as u16,
        );

        packet.push_header(&header);

        self.netdev
            .send_packet(
                self.arp_cache.lock().lookup(address).unwrap(),
                FrameType::IPv4,
                packet,
            )
            .await
    }

    pub async fn serve(self: Arc<Self>) -> Result<(), ErrorType> {
        loop {
            let packet = self.netdev.read_packet().await?;
            let mut packet = Packet::new(packet);
            let eth = packet.parse_mac_header::<EthHeader>()?;
            let source = eth.source();
            let frame_type = eth.frame_type()?;

            let decision = match frame_type {
                FrameType::ARP => self.arp_cache.lock().handle(self.clone(), packet),
                FrameType::IPv4 => inet::handle(self.clone(), packet).await,
                FrameType::IPv6 => todo!(),
            }?;

            match decision {
                PacketDecision::Reply(packet) => {
                    self.netdev.send_packet(source, frame_type, packet).await?;
                }
                PacketDecision::Drop => println!("Packet drop!"),
            }
        }
    }
}
