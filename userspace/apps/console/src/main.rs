#![no_main]
#![no_std]

use libc::handle::Handle;
use libc::{main, port::Port};

mod console;

#[main]
fn main(root: Handle) {
    let client = bindings_NameServer::NameServer::new(Port::new(root));
    let serial_backend = client
        .Get("serial")
        .expect("Failed to find serial backend")
        .handle;

    let serial_backend = Port::new(serial_backend);
    let serial_backend = bindings_Serial::Serial::new(serial_backend);

    console::Console::new(serial_backend).serve();
}
include!(concat!(env!("OUT_DIR"), "/hello.rs"));
include!(concat!(env!("OUT_DIR"), "/serial.rs"));
