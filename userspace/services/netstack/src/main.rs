#![no_std]
#![no_main]
#![feature(variant_count)]

use alloc::sync::Arc;
use bindings_NameServer::NameServer;
use libc::handle::Handle;
use net::ipv4::{IPv4, Ipv4Config};
use netdev::{Netdev, Nic};
use netstack::NetStack;
use rokio::port::Port;
use rtl::error::ErrorType;

mod netdev;
mod netstack;
mod packet;
mod socket;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let ns = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let nic = Nic::new(&ns).await?;

    println!("Hello, net!");

    let netdev = Netdev::new(
        nic,
        Ipv4Config {
            address: IPv4::new(192, 168, 100, 2),
            prefix_len: 24,
            gateway: Some(IPv4::new(192, 168, 100, 1)),
        },
    )
    .await?;

    let netdev = Arc::new(netdev);
    let netstack = Arc::new(NetStack::new(netdev));

    netstack::init();
    rokio::executor::spawn(netstack.clone().serve());
    netstack::serve(netstack, ns).await?;

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
