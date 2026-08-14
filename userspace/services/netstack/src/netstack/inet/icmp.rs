use super::{register_proto, InetProtocol};
use crate::netstack::{NetStack, PacketDecision};
use crate::packet::Packet;
use crate::socket::Socket;
use crate::socket::SocketOps;
use adt::idalloc::IdAllocator;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use net::checksum::checksum;
use net::ipv4::{
    icmp::{IcmpEchoHeader, IcmpHeader, PacketType},
    Protocol,
};
use netstack_types::icmp::EchoRequest;
use rtl::error::ErrorType;
use spin::Mutex;
use zerocopy::FromBytes;

type IcmpSocketArc = Arc<Socket<IcmpSocket>>;

struct IcmpInner {
    map: BTreeMap<u16, IcmpSocketArc>,
    alloc: IdAllocator<{ 1 << 16 }>,
}

struct Icmp(Mutex<IcmpInner>);

impl IcmpInner {
    const fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            alloc: IdAllocator::new(),
        }
    }

    fn socket(&self, id: u16) -> Option<&IcmpSocketArc> {
        self.map.get(&id)
    }

    fn allocate_id(&mut self, socket: IcmpSocketArc) -> Option<u16> {
        let id = self.alloc.allocate()?;

        self.map.insert(id as u16, socket);
        Some(id as u16)
    }
}

static ICMP: Icmp = Icmp(Mutex::new(IcmpInner::new()));

pub fn init() {
    register_proto(&ICMP, Protocol::ICMP);
}

#[async_trait::async_trait]
impl InetProtocol for Icmp {
    async fn receive(&self, mut packet: Packet) -> Result<PacketDecision, ErrorType> {
        let destination = packet.network_header::<net::ipv4::IPv4Header>().source();
        let icmp = IcmpHeader::mut_from_prefix(packet.payload_mut())
            .map_err(|_| ErrorType::BufferTooSmall)?
            .0;

        println!("recv packet {:?}", icmp.packet_type());

        match icmp.packet_type()? {
            PacketType::EchoRequest => {
                icmp.set_packet_type(PacketType::EchoReply);
                icmp.set_checksum(0);

                let crc = checksum(packet.payload());

                let icmp = IcmpHeader::mut_from_prefix(packet.payload_mut())
                    .map_err(|_| ErrorType::BufferTooSmall)?
                    .0;
                icmp.set_checksum(crc);

                Ok(PacketDecision::TransmitIp {
                    destination,
                    packets: vec![packet],
                })
            }
            PacketType::EchoReply => {
                let icmp = IcmpEchoHeader::ref_from_prefix(
                    &packet.payload_mut()[core::mem::size_of::<IcmpEchoHeader>()..],
                )
                .map_err(|_| ErrorType::BufferTooSmall)?
                .0;

                match ICMP.0.lock().socket(icmp.id()) {
                    Some(socket) => {
                        socket.push_packet(packet);

                        Ok(PacketDecision::Handled)
                    }
                    _ => Ok(PacketDecision::Handled),
                }
            }
        }
    }
}

pub struct IcmpSocket {
    id: Mutex<Option<u16>>,
}

#[async_trait::async_trait]
impl SocketOps for IcmpSocket {
    fn new() -> Self {
        Self {
            id: Mutex::new(None),
        }
    }

    async fn send_to(
        &self,
        sock: &Arc<Socket<Self>>,
        netstack: &Arc<NetStack>,
        address: net::ipv4::IPv4,
        data: &[u8],
    ) -> Result<(), ErrorType> {
        let (request, payload) =
            EchoRequest::ref_from_prefix(data).map_err(|_| ErrorType::BufferTooSmall)?;

        let mut id = self.id.lock();

        let id = match *id {
            Some(id) => id,
            None => {
                let new = ICMP.0.lock().allocate_id(sock.clone()).unwrap();

                *id = Some(new);
                new
            }
        };

        let mut packet = Packet::with_capacity(128, payload.len());
        let echo = IcmpEchoHeader::new(id, request.sequence());
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
