use super::{InetProtocol, register_proto};
use crate::netstack::{NetStack, PacketDecision};
use crate::packet::Packet;
use crate::socket::Socket;
use crate::socket::SocketOps;
use alloc::boxed::Box;
use alloc::sync::Arc;
use net::checksum::checksum;
use net::ipv4::{
    Protocol,
    icmp::{IcmpEchoHeader, IcmpHeader, PacketType},
};
use netstack_types::icmp::EchoRequest;
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

pub struct IcmpSocket;

#[async_trait::async_trait]
impl SocketOps for IcmpSocket {
    async fn send_to(
        &self,
        _sock: &Arc<Socket<Self>>,
        netstack: &Arc<NetStack>,
        address: net::ipv4::IPv4,
        data: &[u8],
    ) -> Result<(), ErrorType> {
        let (request, payload) =
            EchoRequest::ref_from_prefix(data).map_err(|_| ErrorType::BufferTooSmall)?;

        let mut packet = Packet::with_capacity(128, payload.len());
        let echo = IcmpEchoHeader::new(request.identifier(), request.sequence());
        let header = IcmpHeader::new(PacketType::EchoRequest as _, 0);

        packet.push_payload(payload);
        packet.push_header(&echo);
        packet.push_header(&header);

        let crc = checksum(packet.payload());
        let header = IcmpHeader::mut_from_prefix(packet.payload_mut())
            .map_err(|_| ErrorType::BufferTooSmall)?
            .0;

        header.set_checksum(crc);

        netstack
            .send_packet_to(address, Protocol::ICMP, packet)
            .await
    }

    async fn receive(
        &self,
        _sock: &Arc<Socket<Self>>,
        _netstack: &Arc<NetStack>,
        _data: &mut [u8],
    ) -> Result<(), ErrorType> {
        todo!()
    }
}
