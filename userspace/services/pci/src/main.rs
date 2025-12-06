#![no_std]
#![no_main]

use libc::{main, handle::Handle};

#[main]
fn main(_nameserver: Handle) {
    println!("Hello, world!");
}
