use super::nic::Nic;
use crate::arp::ArpCache;
use net::eth::{
    arp::{Arp, ArpOperation},
    frame::{Frame, FrameType},
    ipv4::{IPv4, IPv4Out, Protocol},
    mac::Mac,
};
use net::ip::icmp::Icmp;
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

    pub fn handle_arp<'a>(&mut self, frame: &'a Frame) -> Result<Option<Arp>, ErrorType> {
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

            Ok(Some(arp_reply))
        } else {
            Ok(None)
        }
    }

    pub fn handle_icmp<'a>(&mut self, packet: &IPv4<'a>) -> Result<Option<Icmp<'a>>, ErrorType> {
        let icmp = packet.payload::<Icmp>()?;

        match icmp {
            Icmp::EchoRequest { id, seq, payload } => {
                Ok(Some(Icmp::EchoReply { id, seq, payload }))
            }
            e => todo!("{e:?}"),
        }
    }

    pub fn handle_ipv4<'a>(
        &mut self,
        frame: &'a Frame,
    ) -> Result<Option<IPv4Out<Icmp<'a>>>, ErrorType> {
        let ipv4 = frame.payload::<IPv4>()?;
        let my_ip = self.config.address;

        if ipv4.destination() != my_ip {
            return Ok(None);
        }

        let reply = match ipv4.protocol() {
            Protocol::ICMP => self.handle_icmp(&ipv4),
        }?;

        Ok(reply.map(|payload| IPv4Out::new(ipv4.source(), my_ip, payload)))
    }

    pub async fn serve(mut self) -> Result<(), ErrorType> {
        loop {
            let packet = self.nic.read_packet().await?;
            let frame: Frame = packet.as_slice().try_into().unwrap();
            let source = frame.source();
            let mac = self.mac;

            let reply = match frame.frame_type() {
                FrameType::ARP => self
                    .handle_arp(&frame)?
                    .map(|reply| Frame::serialize(source, mac, reply)),
                FrameType::IPv4 => self
                    .handle_ipv4(&frame)?
                    .map(|reply| Frame::serialize(source, mac, reply)),
                e => todo!("{e:?}"),
            };

            if let Some(reply) = reply {
                self.nic.send_packet(&reply).await?;
            }
        }
    }
}
