/// ARP handler
use super::neighbor::Neighbor;
use crate::netdev::Netdev;
use crate::netstack::{NetStack, PacketDecision};
use crate::packet::Packet;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use net::ethernet::arp::{ArpHeader, ArpOperation, ArpPayload};
use net::ethernet::{FrameType, Mac};
use net::ipv4::IPv4;
use rtl::error::ErrorType;
use zerocopy::FromBytes;

#[derive(Default)]
pub struct NeighborCache {
    neighbors: BTreeMap<IPv4, Neighbor>,
}

pub enum NeighborResult {
    Send { mac: Mac, packet: Packet },
    Resolve(Packet),
}

impl NeighborCache {
    pub fn send(
        &mut self,
        iface: &Arc<Netdev>,
        destination: IPv4,
        packet: Packet,
    ) -> Result<NeighborResult, ErrorType> {
        let entry = self
            .neighbors
            .entry(destination)
            .or_insert(Neighbor::new_incomplete(destination));

        entry.send(iface, packet)
    }

    pub fn handle(
        &mut self,
        netstack: Arc<NetStack>,
        mut packet: Packet,
    ) -> Result<PacketDecision, ErrorType> {
        let header = packet.parse_network_header_mut::<ArpHeader>()?;

        match header.operation()? {
            ArpOperation::Request => {
                header.set_operation(ArpOperation::Reply);

                let arp = ArpPayload::mut_from_prefix(packet.payload_mut())
                    .map_err(|_| ErrorType::BufferTooSmall)?
                    .0;

                self.neighbors.insert(
                    arp.sender_ip,
                    Neighbor::new_reachable(arp.sender_ip, arp.sender_mac),
                );

                if arp.target_ip.is_anycast() || arp.target_ip == netstack.ip_address() {
                    arp.target_ip = arp.sender_ip;
                    arp.target_mac = arp.sender_mac;

                    arp.sender_ip = netstack.ip_address();
                    arp.sender_mac = netstack.mac_address();

                    Ok(PacketDecision::TransmitEthernet {
                        destination: arp.target_mac,
                        frame_type: FrameType::ARP,
                        packets: vec![packet],
                    })
                } else {
                    Ok(PacketDecision::Drop)
                }
            }
            ArpOperation::Reply => {
                let arp = ArpPayload::mut_from_prefix(packet.payload_mut())
                    .map_err(|_| ErrorType::BufferTooSmall)?
                    .0;

                println!("{:?}", arp);

                match self.neighbors.get_mut(&arp.sender_ip) {
                    Some(entry) => match entry.complete(arp.sender_mac) {
                        Err(_) => Ok(PacketDecision::Drop),
                        Ok(Some(pending)) => Ok(PacketDecision::TransmitEthernet {
                            destination: arp.sender_mac,
                            frame_type: FrameType::IPv4,
                            packets: pending.into_iter().collect(),
                        }),
                        Ok(None) => Ok(PacketDecision::Handled),
                    },
                    _ => Ok(PacketDecision::Drop),
                }
            }
        }
    }
}
