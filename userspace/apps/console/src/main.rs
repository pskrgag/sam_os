#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use rtl::handle::Handle;
use interfaces::implementation::nameserver;
use interfaces::implementation::serial;

mod console;
mod backend;
mod commands;

use backend::uart::UartBackend;

#[main]
fn main(boot_handle: Handle) {
    nameserver::init(boot_handle);

    let p = nameserver::find_service("serial").unwrap();

    serial::init(p);
    println!("Starting console app.....");
    let c = console::Console::<UartBackend>::new();
    c.exec();
}
