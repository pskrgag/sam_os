#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use shared::vmm::MappingType;

mod cpio;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

#[main]
fn main() {
    println!("Init proccess started");

    let cpio = cpio::Cpio::new(CPIO).unwrap();
    let a = libc::vmm::vm_allocate(0x1000, MappingType::UserData);

    for i in cpio.iter() {
        println!("{:?}", i);

        let _elf = i.data();
    }
}
