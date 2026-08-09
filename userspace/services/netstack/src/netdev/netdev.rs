use super::nic::Nic;
use alloc::vec::Vec;
use net::ethernet::Mac;
use net::ipv4::{IPv4, Ipv4Config};
use rtl::error::ErrorType;

pub struct Netdev {
    nic: Nic,
    mac: Mac,
    config: Ipv4Config,
}

impl Netdev {
    pub async fn new(nic: Nic, config: Ipv4Config) -> Result<Self, ErrorType> {
        let mac = nic.mac().await?;

        Ok(Self { nic, mac, config })
    }

    pub fn ip_address(&self) -> IPv4 {
        self.config.address
    }

    pub fn mac_address(&self) -> Mac {
        self.mac
    }

    pub async fn read_packet(&self) -> Result<Vec<u8>, ErrorType> {
        self.nic.read_packet().await
    }

    pub async fn send_packet(&self, packet: &[u8]) -> Result<(), ErrorType> {
        self.nic.send_packet(packet).await
    }
}
