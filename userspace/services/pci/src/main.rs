#![no_std]
#![no_main]

use alloc::boxed::Box;
use bindings_NameServer::NameServer;
use bindings_Pci::Pci;
use dispatch_loop::EndpointsDispatcher;
use fdt::Fdt;
use libc::{factory::factory, handle::Handle, main, port::Port, syscalls::Syscall};
use rtl::locking::spinlock::Spinlock;

mod dispatcher;
mod ecam;

pub static DISPATH_POOL: Spinlock<EndpointsDispatcher> = Spinlock::new(EndpointsDispatcher::new());

#[main]
fn main(nameserver: Handle) {
    let fdt = Syscall::get_fdt().unwrap();
    let fdt = unsafe { Fdt::from_ptr(fdt.to_raw::<u8>()).unwrap() };
    let ecam = ecam::PciEcam::new(&fdt).unwrap();
    let port = factory().create_port().unwrap();

    ecam.enumerate();

    let ns = NameServer::new(Port::new(nameserver));

    ns.Register("pci", port.handle())
        .expect("Failed to register PCI handle");

    let pci = Pci::new(port, ());

    DISPATH_POOL.lock().add(Box::new(pci));
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/pci.rs"));
