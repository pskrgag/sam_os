#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;

#[main]
fn main() {
    println!("Hello, world!");

    loop {
        for _ in 0..100000 {
            println!("YAy! 2");
            libc::syscalls::Syscall::sys_yield();
        }
    }
}
