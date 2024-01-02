#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use rtl::handle::{Handle, HANDLE_INVALID};
use libc::port::Port;

#[main]
fn main(boot_handle: Handle) {

    assert!(boot_handle != HANDLE_INVALID);

    let p = Port::new(boot_handle);
    p.send();

    println!("Hello, world!");
}
