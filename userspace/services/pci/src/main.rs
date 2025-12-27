#![no_std]
#![no_main]

use alloc::boxed::Box;
use bindings_NameServer::NameServer;
use bindings_Pci::{DeviceRx, DeviceTx, Pci};
use device::PciDevice;
use dispatch_loop::EndpointsDispatcher;
use fdt::Fdt;
use hal::address::VirtualAddress;
use libc::{factory::factory, handle::Handle, main, port::Port, syscalls::Syscall};
use rtl::locking::spinlock::Spinlock;

mod device;
mod ecam;

pub static DISPATH_POOL: EndpointsDispatcher = EndpointsDispatcher::new();

#[main]
fn main(nameserver: Handle) {
    let fdt = Syscall::get_fdt().unwrap();
    let fdt = unsafe { Fdt::from_ptr(fdt.to_raw::<u8>()).unwrap() };
    let ecam = ecam::PciEcam::new(&fdt).unwrap();
    let port = factory().create_port().unwrap();

    let ns = NameServer::new(Port::new(nameserver));

    ns.Register("pci", port.handle())
        .expect("Failed to register PCI handle");

    let pci =
        Pci::new(port, Spinlock::new(ecam)).register_handler(|d: DeviceTx, ecam| {
            let (disp, handle) = PciDevice::new(d.vendor, d.device, ecam.clone())?;

            DISPATH_POOL.add(disp);
            Ok(DeviceRx { handle })
        });

    DISPATH_POOL.add(Box::new(pci));
    DISPATH_POOL.dispatch().unwrap();
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
