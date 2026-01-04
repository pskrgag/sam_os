#![no_std]
#![no_main]
#![allow(unexpected_cfgs)]

use bindings_Device::Device;
use bindings_NameServer::NameServer;
use bindings_Pci::Pci;
use hal::{address::MemRange, arch::PAGE_SIZE};
use libc::{handle::Handle, vmm::vms::vms};
use rokio::port::Port;

mod regs;
mod sdhci;
mod server;

#[rokio::main]
async fn main(nameserver: Option<Handle>) {
    let ns = NameServer::new(unsafe { Port::new(nameserver.unwrap()) });
    let pci = ns.Get("pci".try_into().unwrap()).await.unwrap();
    let pci = unsafe { Pci::new(Port::new(pci.handle)) };

    // These IDS are from QEMU
    let pci_handle =
        Device::new(unsafe { Port::new(pci.Device(0x1b36, 0x7).await.unwrap().handle) });

    let res = pci_handle.Map().await.unwrap();
    assert_eq!(res.data.len(), 1);

    let va = vms()
        .map_phys(MemRange::new(
            (res.data[0].base as usize).into(),
            (res.data[0].size as usize).next_multiple_of(PAGE_SIZE),
        ))
        .unwrap();
    let mut sdhci = sdhci::Sdhci::new(va).unwrap();
    println!("SDHCI version {:?}", sdhci.version());

    server::start_server(sdhci, &ns).await.unwrap();
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
