#![no_std]
#![no_main]
#![feature(variant_count)]

use alloc::sync::Arc;
use bindings_NameServer::NameServer;
use libc::handle::Handle;
use net::ipv4::{IPv4, Ipv4Config};
use netstack::Interface;
use rokio::port::Port;
use rtl::error::ErrorType;

mod arp;
mod inet;
mod netstack;
mod nic;
mod packet;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let ns = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let nic = nic::Nic::new(&ns).await?;

    println!("Hello, net!");

    let iface = Interface::new(
        nic,
        Ipv4Config {
            address: IPv4::new(192, 168, 100, 2),
            prefix_len: 24,
            gateway: Some(IPv4::new(192, 168, 100, 1)),
        },
    )
    .await?;

    inet::init();

    Arc::new(iface).serve().await?;
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
