#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(thread_local)]

use alloc::string::ToString;
use libc::main;
use libc::port::Port;
use libc::task::Task;
use rtl::handle::Handle;
use rtl::handle::HANDLE_INVALID;

use rtl::cpio::Cpio;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main(boot_handle: Handle) {
    println!("Init proccess started");

    assert!(boot_handle == HANDLE_INVALID);

    let cpio = Cpio::new(CPIO).unwrap();

    let p = Port::create().unwrap();

    for i in cpio.iter() {
        println!("{:?}", i);

        let elf = i.data();
        let mut task =
            Task::create_from_elf(elf, "test task".to_string()).expect("Failed to create task");
        task.start(p.handle()).unwrap();

        println!("Spawned '{}'", task.name())
    }

    let mut msg = [111u8; 10];

    p.receive_data(&mut msg);
    println!("HELLO {:?}", msg);
}
