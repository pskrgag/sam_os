use super::inet;
use crate::netdev::{
    neighbor::{NeighborCache, NeighborResult},
    Netdev,
};
use crate::packet::Packet;
use alloc::sync::Arc;
use alloc::vec::Vec;
use net::ethernet::{EthHeader, FrameType, Mac};
use net::ipv4::{IPv4, IPv4Header, Protocol};
use rtl::error::ErrorType;
use spin::Mutex;

#[derive(Debug)]
pub enum PacketDecision {
    TransmitIp {
        destination: IPv4,
        packets: Vec<Packet>,
    },
    TransmitEthernet {
        destination: Mac,
        frame_type: FrameType,
        packets: Vec<Packet>,
    },
    Handled,
    Drop,
}

pub struct NetStack {
    netdev: Arc<Netdev>,
    neighbor_cache: Mutex<NeighborCache>,
}

impl NetStack {
    pub fn new(netdev: Arc<Netdev>) -> Self {
        Self {
            netdev,
            neighbor_cache: Mutex::new(NeighborCache::default()),
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

        // TODO: actually handle local sends.

        packet.push_header(&header);

        self.transmit_ip(address, packet).await
    }

    async fn transmit_ip(&self, destination: IPv4, packet: Packet) -> Result<(), ErrorType> {
        let result = self
            .neighbor_cache
            .lock()
            .send(&self.netdev, destination, packet)?;

        match result {
            NeighborResult::Send { mac, packet } => {
                // println!("Sent packet {:?}", FrameType::IPv4);
                self.netdev.send_packet(mac, FrameType::IPv4, packet).await
            }
            NeighborResult::Resolve(packet) => {
                // println!("Sent packet {:?}", FrameType::ARP);
                self.netdev
                    .send_packet(Mac::broadcast(), FrameType::ARP, packet)
                    .await
            }
        }
    }

    pub async fn serve(self: Arc<Self>) -> Result<(), ErrorType> {
        loop {
            let packet = self.netdev.read_packet().await?;
            let mut packet = Packet::new(packet);
            let eth = packet.parse_mac_header::<EthHeader>()?;
            let frame_type = eth.frame_type()?;

            // println!("Recv packet {:?}", frame_type);

            let decision = match frame_type {
                FrameType::ARP => self.neighbor_cache.lock().handle(self.clone(), packet),
                FrameType::IPv4 => inet::handle(self.clone(), packet).await,
                FrameType::IPv6 => todo!(),
            }?;

            match decision {
                PacketDecision::TransmitIp {
                    destination,
                    packets,
                } => {
                    for packet in packets {
                        self.transmit_ip(destination, packet).await?;
                    }
                }
                PacketDecision::TransmitEthernet {
                    destination,
                    frame_type,
                    packets,
                } => {
                    for packet in packets {
                        self.netdev
                            .send_packet(destination, frame_type, packet)
                            .await?;
                    }
                }
                PacketDecision::Handled => {}
                PacketDecision::Drop => println!("Packet drop!"),
            }
        }
    }
}
