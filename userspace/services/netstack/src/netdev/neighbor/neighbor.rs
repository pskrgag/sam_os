use super::cache::NeighborResult;
use crate::netdev::netdev::Netdev;
use crate::packet::Packet;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use net::ethernet::{arp::*, Mac};
use net::ipv4::IPv4;
use rtl::error::ErrorType;
use zerocopy::IntoBytes;

enum NeighborState {
    Incomplete(VecDeque<Packet>),
    Reachable(Mac),
}

pub(crate) struct Neighbor {
    address: IPv4,
    state: NeighborState,
}

impl Neighbor {
    pub fn new_incomplete(address: IPv4) -> Self {
        Self {
            address,
            state: NeighborState::Incomplete(VecDeque::new()),
        }
    }

    pub fn new_reachable(address: IPv4, mac: Mac) -> Self {
        Self {
            address,
            state: NeighborState::Reachable(mac),
        }
    }

    pub fn complete(&mut self, mac: Mac) -> Result<Option<VecDeque<Packet>>, ErrorType> {
        match &mut self.state {
            NeighborState::Reachable(m) if &mac == m => Ok(None),
            NeighborState::Reachable(_) => Err(ErrorType::AlreadyExists),
            NeighborState::Incomplete(pending) => {
                let result = pending.drain(..).collect();

                self.state = NeighborState::Reachable(mac);
                Ok(Some(result))
            }
        }
    }

    pub fn send(
        &mut self,
        iface: &Arc<Netdev>,
        packet: Packet,
    ) -> Result<NeighborResult, ErrorType> {
        match &mut self.state {
            NeighborState::Reachable(mac) => Ok(NeighborResult::Send { mac: *mac, packet }),
            NeighborState::Incomplete(pending) => {
                let mut arp = Packet::with_capacity(128, 128);
                let header = ArpHeader::new(
                    ArpHardware::Ethernet,
                    ArpProtocol::IPv4,
                    ArpOperation::Request,
                );
                let payload = ArpPayload {
                    sender_mac: iface.mac_address(),
                    sender_ip: iface.ip_address(),
                    target_mac: Mac::broadcast(),
                    target_ip: self.address,
                };

                arp.push_payload(payload.as_bytes());
                arp.push_header(&header);

                pending.push_back(packet);
                Ok(NeighborResult::Resolve(arp))
            }
        }
    }
}
