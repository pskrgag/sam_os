#![no_std]
#![no_main]

use interfaces::implementation::nameserver;
use libc::main;
use libc::port::Port;
use rtl::handle::{Handle, HANDLE_INVALID};

mod serial;

#[main]
fn main(boot_handle: Handle) {
    assert!(boot_handle != HANDLE_INVALID);

    let p = Port::create().unwrap();

    nameserver::init(boot_handle);
    nameserver::register_service("serial", p.handle()).unwrap();

    serial::start_serial(p);
}
