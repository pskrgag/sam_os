use super::nic::Nic;
use crate::arp::ArpCache;
use alloc::vec::Vec;
use net::eth::{
    arp::{Arp, ArpOperation},
    frame::{Frame, FrameType},
    mac::Mac,
};
use net::ip::v4::Ipv4Config;
use rtl::error::ErrorType;

pub struct Interface {
    nic: Nic,
    mac: Mac,
    config: Ipv4Config,
    arp_cache: ArpCache,
}

impl Interface {
    pub async fn new(nic: Nic, config: Ipv4Config) -> Result<Self, ErrorType> {
        let mac = nic.mac().await?;

        Ok(Self {
            nic,
            mac,
            config,
            arp_cache: ArpCache::default(),
        })
    }

    pub fn handle_arp(&mut self, frame: Frame) -> Result<Option<Vec<u8>>, ErrorType> {
        let arp = frame.payload::<Arp>()?;

        self.arp_cache.insert(arp.sender_ip(), arp.sender_mac());

        if arp.target_ip().is_anycast() || arp.target_ip() == self.config.address {
            let arp_reply = Arp::new(
                ArpOperation::Reply,
                self.mac,
                self.config.address,
                arp.sender_mac(),
                arp.sender_ip(),
            );

            Ok(Some(Frame::serialize(frame.source(), self.mac, arp_reply)))
        } else {
            Ok(None)
        }
    }

    pub async fn serve(mut self) -> Result<(), ErrorType> {
        loop {
            let packet = self.nic.read_packet().await?;
            let frame: Frame = packet.as_slice().try_into().unwrap();

            let reply = match frame.frame_type() {
                FrameType::ARP => self.handle_arp(frame)?,
                _ => todo!(),
            };

            if let Some(reply) = reply {
                self.nic.send_packet(&reply).await?;
            }
        }
    }
}
