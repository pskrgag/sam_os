#![no_std]
#![no_main]
#![feature(format_args_nl)]

use interfaces::implementation::nameserver;
use interfaces::implementation::serial;
use libc::main;
use rtl::handle::Handle;

mod backend;
mod commands;
mod console;

use backend::uart::UartBackend;

#[main]
fn main(boot_handle: Handle) {
    nameserver::init(boot_handle);

    let mut p;

    // Wait until serial wakes. Smells a bit, but leave as-is for now
    while {
        p = nameserver::find_service("serial");
        if p.is_err() {
            libc::syscalls::Syscall::sys_yield();
        }

        p.is_err()
    } {}

    serial::init(p.unwrap());
    println!("Starting console app");
    let c = console::Console::<UartBackend>::new();
    c.exec();
}
