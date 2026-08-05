use super::{InetProtocol, register_proto};
use crate::netstack::PacketDecision;
use crate::packet::Packet;
use alloc::boxed::Box;
use net::checksum::checksum;
use net::ipv4::{
    Protocol,
    icmp::{IcmpHeader, PacketType},
};
use rtl::error::ErrorType;
use zerocopy::FromBytes;

pub struct ICMP;

pub fn init() {
    register_proto(&ICMP, Protocol::ICMP);
}

#[async_trait::async_trait]
impl InetProtocol for ICMP {
    async fn handle(&self, mut packet: Packet) -> Result<PacketDecision, ErrorType> {
        let icmp = IcmpHeader::mut_from_prefix(packet.payload_mut())
            .map_err(|_| ErrorType::BufferTooSmall)?
            .0;

        match icmp.packet_type()? {
            PacketType::EchoRequest => {
                icmp.set_packet_type(PacketType::EchoReply);
                icmp.set_checksum(0);

                let crc = checksum(packet.payload());

                let icmp = IcmpHeader::mut_from_prefix(packet.payload_mut())
                    .map_err(|_| ErrorType::BufferTooSmall)?
                    .0;
                icmp.set_checksum(crc);

                Ok(PacketDecision::Reply(packet))
            }
            e => todo!("{e:?}"),
        }
    }
}
