#![no_main]
#![no_std]

use alloc::string::ToString;
use libc::{main, task::Task};

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main(_: Handle) {
    let p = Port::create().unwrap();

    for i in cpio_reader::iter_files(CPIO) {
        let elf = i.file();
        let name = if let Some(pos) = i.name().rfind('/') {
            &i.name()[pos + 1..]
        } else {
            i.name()
        };

        println!("Creating");
        let mut task = Task::create_from_elf(elf, name.to_string()).expect("Failed to create task");
        task.start(p.handle()).unwrap();

        println!("Spawned '{}'", task.name())
    }
    println!("Hello, world!");
}

include!(concat!(env!("OUT_DIR"), "/hello.rs"));
