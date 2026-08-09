/// ARP handler
use crate::netstack::{NetStack, PacketDecision};
use crate::packet::Packet;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use net::ethernet::{
    Mac,
    arp::{ArpOperation, ArpPayload},
};
use net::ipv4::IPv4;
use rtl::error::ErrorType;
use zerocopy::FromBytes;

#[derive(Default)]
pub struct ArpCache {
    cache: BTreeMap<IPv4, Mac>,
}

impl ArpCache {
    pub fn insert(&mut self, ip: IPv4, mac: Mac) {
        self.cache.insert(ip, mac);
    }

    pub fn lookup(&mut self, ip: IPv4) -> Option<Mac> {
        self.cache.get(&ip).cloned()
    }

    pub fn handle(
        &mut self,
        netstack: Arc<NetStack>,
        mut packet: Packet,
    ) -> Result<PacketDecision, ErrorType> {
        let header = packet.parse_network_header_mut::<net::ethernet::arp::ArpHeader>()?;

        match header.operation()? {
            ArpOperation::Request => {}
            ArpOperation::Reply => todo!(),
        }

        header.set_operation(ArpOperation::Reply);

        let arp = ArpPayload::mut_from_prefix(packet.payload_mut())
            .map_err(|_| ErrorType::BufferTooSmall)?
            .0;

        self.cache.insert(arp.sender_ip, arp.sender_mac);

        if arp.target_ip.is_anycast() || arp.target_ip == netstack.ip_address() {
            arp.target_ip = arp.sender_ip;
            arp.target_mac = arp.sender_mac;

            arp.sender_ip = netstack.ip_address();
            arp.sender_mac = netstack.mac_address();

            Ok(PacketDecision::Reply(packet))
        } else {
            Ok(PacketDecision::Drop)
        }
    }
}
