#![no_std]
#![no_main]

use alloc::boxed::Box;
use bindings_NameServer::NameServer;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

mod e1000;
mod driver;
mod server;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let ns = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let e1000 = e1000::E1000::new(&ns).await?;

    println!("Starting nic...");

    server::start_server(Box::new(e1000), ns).await
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
