#![no_std]
#![no_main]

use bindings_BlkDev::BlkDev;
use bindings_NameServer::NameServer;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let nameserver = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let root = nameserver.Get("blkdev".try_into().unwrap()).await?;
    let root = BlkDev::new(unsafe { Port::new(root.handle) });

    println!("Hello, world!");
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
