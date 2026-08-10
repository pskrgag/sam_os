#![no_main]
#![no_std]

use libc::handle::Handle;
use rokio::port::Port;

mod commands;
mod console;
mod cwd;

#[rokio::main]
async fn main(root: Option<Handle>) {
    let nameserver = bindings_NameServer::NameServer::new(unsafe { Port::new(root.unwrap()) });

    let serial = nameserver.Get("serial".into()).await.unwrap().handle;
    let vfs = nameserver.Get("vfs".into()).await.unwrap().handle;
    let netstack = nameserver.Get("netstack".into()).await.unwrap().handle;
    let serial_backend = bindings_Serial::Serial::new(unsafe { Port::new(serial) });

    unsafe {
        fs::init(vfs).await.unwrap();
        socket::init(netstack);
    }

    println!("Starting console...");
    console::Console::new(serial_backend).serve().await;
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/serial.rs"));
include!(concat!(env!("OUT_DIR"), "/vfs.rs"));
