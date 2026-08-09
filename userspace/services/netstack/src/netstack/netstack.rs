use super::inet;
use crate::netdev::{Netdev, arp::ArpCache};
use crate::packet::Packet;
use alloc::sync::Arc;
use net::ethernet::{EthHeader, FrameType, Mac};
use net::ipv4::IPv4;
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

    pub async fn serve(self: Arc<Self>) -> Result<(), ErrorType> {
        loop {
            let packet = self.netdev.read_packet().await?;
            let mut packet = Packet::new(packet);
            let frame_type = packet.parse_mac_header::<EthHeader>()?.frame_type()?;

            let decision = match frame_type {
                FrameType::ARP => self.arp_cache.lock().handle(self.clone(), packet),
                FrameType::IPv4 => inet::handle(self.clone(), packet).await,
                FrameType::IPv6 => todo!(),
            }?;

            match decision {
                PacketDecision::Reply(mut packet) => {
                    packet.mac_header_mut::<EthHeader>().swap_macs();
                    self.netdev.send_packet(&packet.into_data()).await?;
                }
                PacketDecision::Drop => println!("Packet drop!"),
            }
        }
    }
}
