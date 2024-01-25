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

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[allow(redundant_semicolons)]
#[allow(dead_code)]
#[allow(unused_imports)]
mod interface;

mod nameserver;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main(boot_handle: Handle) {
    println!("Nameserver proccess started");

    assert!(boot_handle == HANDLE_INVALID);

    let cpio = Cpio::new(CPIO).unwrap();

    let p = Port::create().unwrap();

    for i in cpio.iter() {
        println!("{:?}", i);

        let elf = i.data();
        let mut task =
            Task::create_from_elf(elf, "task1".to_string()).expect("Failed to create task");
        task.start(p.handle()).unwrap();

        println!("Spawned '{}'", task.name())
    }

    nameserver::start_nameserver(p);
}
