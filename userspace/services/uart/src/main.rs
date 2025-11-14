#![no_main]
#![no_std]

use libc::vmm::vms::vms;
use libc::{handle::Handle, main};
use rtl::vmm::types::{MemRange, PhysAddr, VirtAddr};

mod ecam;

#[main]
fn main(_: Handle) {
    let addr = vms()
        .map_phys(MemRange::new(PhysAddr::new(0x3f000000), 0x01000000))
        .unwrap();

    let ecam = ecam::Ecam::new(MemRange::new(addr, 0x01000000));
    println!("I AM HERE");
}

// include!(concat!(env!("OUT_DIR"), "/hello.rs"));
