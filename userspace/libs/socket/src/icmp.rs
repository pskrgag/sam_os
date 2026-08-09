use crate::bindings_NetStack::Proto;
use crate::socket::SocketProtocol;
use net::ipv4::icmp::IcmpHeader;

pub struct Icmp;

impl SocketProtocol for Icmp {
    type Header = IcmpHeader;

    const PROTO: Proto = Proto::ICMP;
}
