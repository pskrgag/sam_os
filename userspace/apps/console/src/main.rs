#![no_main]
#![no_std]

use libc::handle::Handle;
use rokio::port::Port;

mod console;
mod commands;

#[rokio::main]
async fn main(root: Option<Handle>) {
    let nameserver = bindings_NameServer::NameServer::new(unsafe { Port::new(root.unwrap()) });

    let serial = nameserver
        .Get("serial".try_into().unwrap())
        .await
        .unwrap()
        .handle;
    let vfs = nameserver
        .Get("vfs".try_into().unwrap())
        .await
        .unwrap()
        .handle;
    let serial_backend = bindings_Serial::Serial::new(unsafe { Port::new(serial) });
    let vfs = bindings_Vfs::Vfs::new(unsafe { Port::new(vfs) });

    println!("Starting console...");
    console::Console::new(serial_backend, vfs).serve().await;
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/serial.rs"));
include!(concat!(env!("OUT_DIR"), "/vfs.rs"));
