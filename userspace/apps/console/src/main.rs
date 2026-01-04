#![no_main]
#![no_std]

use libc::handle::Handle;
use rokio::{main, port::Port};

mod console;

#[main]
async fn main(root: Option<Handle>) {
    let nameserver = bindings_NameServer::NameServer::new(unsafe { Port::new(root.unwrap()) });

    let serial = nameserver
        .Get("serial".try_into().unwrap())
        .await
        .unwrap()
        .handle;
    let serial_backend = unsafe { Port::new(serial) };
    let serial_backend = bindings_Serial::Serial::new(serial_backend);

    println!("Starting console...");
    console::Console::new(serial_backend).serve().await;
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/serial.rs"));
