#![no_std]
#![no_main]

use alloc::sync::Arc;
use bindings_NameServer::NameServer;
use bindings_Pci::{Pci, PciRequest};
use device::PciDevice;
use fdt::Fdt;
use hal::address::VirtualAddress;
use libc::{handle::Handle, syscalls::Syscall};
use rokio::port::Port;
use rtl::locking::spinlock::Spinlock;

mod device;
mod ecam;

#[rokio::main]
async fn main(nameserver: Option<Handle>) {
    let fdt = Syscall::get_fdt().unwrap();
    let fdt = unsafe { Fdt::from_ptr(fdt.to_raw::<u8>()).unwrap() };
    let ecam = Arc::new(Spinlock::new(ecam::PciEcam::new(&fdt).unwrap()));
    let port = Port::create().unwrap();

    let ns = NameServer::new(unsafe { Port::new(nameserver.unwrap()) });

    ns.Register("pci".try_into().unwrap(), port.handle())
        .await
        .expect("Failed to register PCI handle");

    Pci::for_each(port, move |req| {
        let ecam = ecam.clone();

        async move {
            match req {
                PciRequest::Device { value, responder } => {
                    let (disp, handle) = PciDevice::new(value.vendor, value.device, ecam.clone())?;

                    rokio::executor::spawn(disp);
                    responder.reply(&handle)?;
                }
            }
            Ok(())
        }
    })
    .await
    .unwrap()
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
