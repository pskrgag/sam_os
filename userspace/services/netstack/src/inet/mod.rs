use super::netstack::{Interface, PacketDecision};
use super::packet::Packet;
use alloc::boxed::Box;
use alloc::sync::Arc;
use net::ipv4::{IPv4Header, Protocol};
use rtl::error::ErrorType;
use spin::Mutex;

pub mod icmp;

#[async_trait::async_trait]
pub trait InetProtocol: Send + Sync {
    async fn handle(&self, packet: Packet) -> Result<PacketDecision, ErrorType>;
}

static PROTOCOLS: Mutex<[Option<Arc<&'static dyn InetProtocol>>; 1000]> =
    Mutex::new([const { None }; 1000]);

pub(crate) fn register_proto(proto_handler: &'static dyn InetProtocol, proto: Protocol) {
    let mut arr = PROTOCOLS.lock();

    assert!(arr[proto as usize].is_none());
    arr[proto as usize] = Some(Arc::new(proto_handler));
}

pub async fn handle(
    iface: Arc<Interface>,
    mut packet: Packet,
) -> Result<PacketDecision, ErrorType> {
    let ipv4 = packet.parse_network_header_mut::<IPv4Header>()?;

    if ipv4.destination() != iface.ip_address() {
        return Ok(PacketDecision::Drop);
    }

    let arr = PROTOCOLS.lock();

    let func = arr[ipv4.protocol()? as usize].as_ref().unwrap().clone();
    drop(arr);

    match func.handle(packet).await? {
        PacketDecision::Reply(mut packet) => {
            let ipv4_mut = packet.network_header_mut::<IPv4Header>();

            ipv4_mut.swap_ips();
            ipv4_mut.checksum();

            Ok(PacketDecision::Reply(packet))
        }
        e => Ok(e),
    }
}

pub fn init() {
    icmp::init();
}
