use super::nic::Nic;
use super::packet::Packet;
use crate::arp::ArpCache;
use alloc::sync::Arc;
use net::ethernet::{EthHeader, FrameType, Mac};
use net::ipv4::{IPv4, Ipv4Config};
use rtl::error::ErrorType;
use spin::Mutex;

pub struct Interface {
    nic: Nic,
    mac: Mac,
    config: Ipv4Config,
    arp_cache: Mutex<ArpCache>,
}

#[derive(Debug)]
pub enum PacketDecision {
    Reply(Packet),
    Drop,
}

impl Interface {
    pub async fn new(nic: Nic, config: Ipv4Config) -> Result<Self, ErrorType> {
        let mac = nic.mac().await?;

        Ok(Self {
            nic,
            mac,
            config,
            arp_cache: Mutex::new(ArpCache::default()),
        })
    }

    pub fn ip_address(&self) -> IPv4 {
        self.config.address
    }

    pub fn mac_address(&self) -> Mac {
        self.mac
    }

    pub async fn serve(self: Arc<Self>) -> Result<(), ErrorType> {
        loop {
            let packet = self.nic.read_packet().await?;
            let mut packet = Packet::new(packet);
            let frame_type = packet.parse_mac_header::<EthHeader>()?.frame_type()?;

            let res = match frame_type {
                FrameType::ARP => self.arp_cache.lock().handle(self.clone(), packet),
                FrameType::IPv4 => super::inet::handle(self.clone(), packet).await,
                FrameType::IPv6 => todo!(""),
            }?;

            match res {
                PacketDecision::Reply(mut packet) => {
                    let header = packet.mac_header_mut::<EthHeader>();

                    header.swap_macs();
                    self.nic.send_packet(&packet.into_data()).await?
                }
                _ => {
                    println!("Packet drop!");
                }
            }
        }
    }
}
