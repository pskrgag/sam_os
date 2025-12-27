#![no_std]
#![no_main]

use bindings_Device::Device;
use bindings_NameServer::NameServer;
use bindings_Pci::Pci;
use hal::{
    address::{MemRange, VirtualAddress},
    arch::PAGE_SIZE,
};
use libc::{handle::Handle, main, port::Port, vmm::vms::vms};

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

    let res = pci_handle.Map().unwrap();
    println!("Hello, world! {:x} {:x}", res.pa, res.size);

    let va = vms()
        .map_phys(MemRange::new(
            (res.pa as usize).into(),
            (res.size as usize).next_multiple_of(PAGE_SIZE),
        ))
        .unwrap();
    unsafe { println!("{:x}", va.to_raw::<u32>().add(0xFE / 4).read_volatile()) };
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
