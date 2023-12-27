#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use libc::task::Task;

use rtl::cpio::Cpio;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main() {
    println!("Init proccess started");

    let cpio = Cpio::new(CPIO).unwrap();

    for i in cpio.iter() {
        println!("{:?}", i);

        let elf = i.data();
        let mut task = Task::create_from_elf(elf).expect("Failed to create task");
        task.start().unwrap();
    }
}
