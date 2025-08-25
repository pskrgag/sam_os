#![no_main]
#![no_std]

use alloc::string::ToString;
use libc::{main, port::Port, task::Task};
use rtl::handle::Handle;

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

        let mut task = Task::create_from_elf(elf, name.to_string()).expect("Failed to create task");
        task.start(p.handle()).unwrap();

        println!("Spawned '{}'", task.name())
    }

    let mut server = bindings::Hello::new(p, ())
        .register_handler(|_: bindings::TestTx, _| Ok(bindings::TestRx { b: 100 }));

    println!("Starting nameserver...");
    server.run().unwrap();
}

include!(concat!(env!("OUT_DIR"), "/hello.rs"));
