#![no_main]
#![no_std]

use libc::main;
use rtl::handle::Handle;

#[main]
fn main(_: Handle) {
    println!("Hello, world!");
}
