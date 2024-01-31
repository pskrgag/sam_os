#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use rtl::handle::{Handle, HANDLE_INVALID};
use interfaces::implementation::nameserver;
use libc::port::Port;

mod serial;

#[main]
fn main(boot_handle: Handle) {
    assert!(boot_handle != HANDLE_INVALID);

    let p = Port::create().unwrap();

    nameserver::init(boot_handle);
    nameserver::register_service("serial", p.handle()).unwrap();

    serial::start_serial(p);
}
