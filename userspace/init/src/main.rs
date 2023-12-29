#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use libc::task::Task;
use alloc::string::ToString;

use rtl::cpio::Cpio;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main() {
    println!("Init proccess started");

    let cpio = Cpio::new(CPIO).unwrap();

    for i in cpio.iter() {
        println!("{:?}", i);

        let elf = i.data();
        let mut task =
            Task::create_from_elf(elf, "test task".to_string()).expect("Failed to create task");
        task.start().unwrap();

        println!("Spawned '{}'", task.name())
    }

    loop {
        for _ in 0..100000 {
            println!("YAy! 1");
            libc::syscalls::Syscall::sys_yield();
        }
    }
}
