#![no_main]
#![no_std]

use alloc::string::ToString;
use libc::{main, port::Port, task::Task, handle::Handle};

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

    let mut server = bindings::NameServer::new(p, ()).register_handler(|t: bindings::RegisterTx, _| {
        println!("{:?}", core::str::from_utf8(&t.name).unwrap());
        Ok(bindings::RegisterRx {})
    });

    println!("Starting nameserver...");
    server.run().unwrap();
}

include!(concat!(env!("OUT_DIR"), "/hello.rs"));
