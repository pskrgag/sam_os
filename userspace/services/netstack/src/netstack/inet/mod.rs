use super::{NetStack, PacketDecision};
use crate::packet::Packet;
use alloc::boxed::Box;
use alloc::sync::Arc;
use net::ipv4::{IPv4Header, Protocol};
use rtl::error::ErrorType;
use spin::Mutex;

pub mod icmp;

#[async_trait::async_trait]
pub trait InetProtocol: Send + Sync {
    async fn receive(&self, packet: Packet) -> Result<PacketDecision, ErrorType>;
}

static PROTOCOLS: Mutex<[Option<Arc<&'static dyn InetProtocol>>; 1000]> =
    Mutex::new([const { None }; 1000]);

pub(crate) fn register_proto(proto_handler: &'static dyn InetProtocol, proto: Protocol) {
    let mut arr = PROTOCOLS.lock();

    assert!(arr[proto as usize].is_none());
    arr[proto as usize] = Some(Arc::new(proto_handler));
}

pub async fn handle(
    netstack: Arc<NetStack>,
    mut packet: Packet,
) -> Result<PacketDecision, ErrorType> {
    let ipv4 = packet.parse_network_header_mut::<IPv4Header>()?;

    if ipv4.destination() != netstack.ip_address() {
        return Ok(PacketDecision::Drop);
    }

    let arr = PROTOCOLS.lock();

    let func = arr[ipv4.protocol()? as usize].as_ref().unwrap().clone();
    drop(arr);

    match func.receive(packet).await? {
        PacketDecision::TransmitIp {
            destination,
            mut packets,
        } => {
            for packet in &mut packets {
                let ipv4 = packet.network_header_mut::<IPv4Header>();

                ipv4.swap_ips();
                ipv4.checksum();
            }

            Ok(PacketDecision::TransmitIp {
                destination,
                packets,
            })
        }
        e => Ok(e),
    }
}

pub fn init() {
    icmp::init();
}
