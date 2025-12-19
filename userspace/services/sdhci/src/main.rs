#![no_std]
#![no_main]

use bindings_NameServer::NameServer;
use libc::{handle::Handle, main, port::Port};

#[main]
fn main(nameserver: Handle) {
    let ns = NameServer::new(Port::new(nameserver));
    let _pci = loop {
        // TODO: add support for loading in dependency
        match ns.Get("pci") {
            Ok(pci) => break pci,
            _ => {}
        }
    };

    println!("Hello, world!");
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
