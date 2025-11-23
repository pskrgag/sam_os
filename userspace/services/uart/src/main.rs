#![no_main]
#![no_std]

// use hal::address::{MemRange, PhysAddr, VirtAddr};
// use libc::vmm::vms::vms;
use libc::{handle::Handle, main};

#[main]
fn main(_: Handle) {
    println!("I AM HERE");
}

// include!(concat!(env!("OUT_DIR"), "/hello.rs"));
