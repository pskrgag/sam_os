#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use libc::vmm::vms::vms;
use rtl::arch::PAGE_SIZE;
use rtl::handle::{Handle, HANDLE_INVALID};
use rtl::uart::*;
use rtl::vmm::types::*;
use libc::port::Port;

// mod transport;
// use transport::*;

#[main]
fn main(boot_handle: Handle) {
    assert!(boot_handle != HANDLE_INVALID);

    let base = vms()
        .map_phys(MemRange::<PhysAddr>::new(0x09000000.into(), PAGE_SIZE))
        .unwrap();

    let mut uart = Uart::init(base);
    let mut b = [1u8; 10];

    let p = Port::new(boot_handle);
    let reply = Port::create().unwrap();

    let mut test = [1u8; 10];
    p.call(&b, Some(&mut test), Some(&reply));

    println!("{:?}", test);
    // init(boot_handle);
    // let mut out = 0;
    // FindService(&12, &mut out);
}
