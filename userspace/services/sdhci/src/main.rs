#![no_std]
#![no_main]

use bindings_Device::Device;
use bindings_NameServer::NameServer;
use bindings_Pci::Pci;
use libc::{handle::Handle, main, port::Port};

#[main]
fn main(nameserver: Handle) {
    let ns = NameServer::new(Port::new(nameserver));
    let _pci = loop {
        // TODO: add support for loading in dependency
        if let Ok(pci) = ns.Get("pci") {
            break pci;
        }
    };

    let pci = Pci::new(Port::new(ns.Get("pci").expect("Failed to get PCI").handle));
    let pci_handle = Device::new(Port::new(pci.Device(0x1b36, 0x7).unwrap().handle));

    pci_handle.Do().unwrap();
    println!("Hello, world!");
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
