#![no_main]
#![no_std]

use libc::{main, port::Port};
use rtl::handle::Handle;

#[main]
fn main(root: Handle) {
    println!("Hello, world!");

    let client = bindings::Hello::new(Port::new(root));
    let res = client.Test(10).unwrap();

    println!("{:?}", res);
}

include!(concat!(env!("OUT_DIR"), "/hello.rs"));
