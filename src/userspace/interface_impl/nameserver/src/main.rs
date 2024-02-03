#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(thread_local)]

use alloc::string::ToString;
use libc::main;
use libc::port::Port;
use libc::task::Task;
use rtl::cpio::Cpio;
use rtl::handle::*;

mod nameserver;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main(boot_handle: Handle) {
    println!("Nameserver proccess started");

    assert!(boot_handle == HANDLE_INVALID);

    let cpio = Cpio::new(CPIO).unwrap();

    let p = Port::create().unwrap();

    for i in cpio.iter() {
        let elf = i.data();
        let name = if let Some(pos) = i.name().rfind('/') {
            &i.name()[pos + 1..]
        } else {
            i.name()
        };

        let mut task =
            Task::create_from_elf(elf, name.to_string()).expect("Failed to create task");
        task.start(p.handle()).unwrap();

        println!("Spawned '{}'", task.name())
    }

    println!("Serving nameserver interface...");
    nameserver::start_nameserver(p);
}
