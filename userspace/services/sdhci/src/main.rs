#![no_std]
#![no_main]
#![allow(unexpected_cfgs)]

use bindings_Device::Device;
use bindings_NameServer::NameServer;
use bindings_Pci::Pci;
use hal::{address::MemRange, arch::PAGE_SIZE};
use libc::{handle::Handle, main, port::Port, vmm::vms::vms};

mod sdhci;
mod regs;

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

    // These IDS are from QEMU
    let pci_handle = Device::new(Port::new(pci.Device(0x1b36, 0x7).unwrap().handle));

    let res = pci_handle.Map().unwrap();
    assert_eq!(res.data.len(), 1);

    let va = vms()
        .map_phys(MemRange::new(
            (res.data[0].base as usize).into(),
            (res.data[0].size as usize).next_multiple_of(PAGE_SIZE),
        ))
        .unwrap();
    let mut sdhci = sdhci::Sdhci::new(va).unwrap();
    println!("SDHCI version {:?}", sdhci.version());

    let mut buffer = [0; 512];
    sdhci.read_block(0, &mut buffer).unwrap();
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
