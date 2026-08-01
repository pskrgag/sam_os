#![no_std]
#![no_main]

use bindings_NameServer::NameServer;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

mod nic;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let ns = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let nic = nic::Nic::new(&ns).await?;

    println!("Hello, net!");

    loop {
        let packet = nic.read_packet().await?;
        println!("received shit");
    }
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
