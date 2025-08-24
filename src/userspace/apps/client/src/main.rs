#![no_main]
#![no_std]

use libc::main;
use rtl::handle::Handle;

#[main]
fn main(root: Handle) {
    println!("Hello, world!");

    let mut client = bindings::Hello::new(root);

    client.Test(10).unwrap();
}

include!(concat!(env!("OUT_DIR"), "/hello.rs"));
