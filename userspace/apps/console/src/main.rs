#![no_main]
#![no_std]

use libc::handle::Handle;
use libc::{main, port::Port};

mod console;

#[main]
fn main(root: Option<Handle>) {
    let nameserver = bindings_NameServer::NameServer::new(unsafe { Port::new(root.unwrap()) });

    let serial = loop {
        // TODO: add support for loading in dependency
        if let Ok(serial) = nameserver.Get("serial") {
            break serial.handle;
        }
    };

    let serial_backend = unsafe { Port::new(serial) };
    let serial_backend = bindings_Serial::Serial::new(serial_backend);

    console::Console::new(serial_backend).serve();
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/serial.rs"));
