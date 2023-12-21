#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;

mod cpio;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main() {
    println!("Init proccess started");

    let cpio = cpio::Cpio::new(CPIO).unwrap();

    for i in cpio.iter() {
        println!("{:?}", i);

        let _elf = i.data();
    }
}
