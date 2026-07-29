#![no_std]
#![no_main]

use bindings_NameServer::NameServer;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

mod e1000;
mod net;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let ns = NameServer::new(unsafe { Port::new(root.unwrap()) });

    let mut e1000 = e1000::E1000::new(ns).await?;

    println!("Starting net...");

    loop {
        let packet = e1000.read_packet();

        if let Some(packet) = packet {
            println!("received");
            e1000.send_packet(&packet);
            println!("sent");
        }
    }

    loop {
        libc::syscalls::Syscall::sys_yield();
    }
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
