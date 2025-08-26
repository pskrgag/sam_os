#![no_main]
#![no_std]

use libc::handle::Handle;
use libc::{main, port::Port};

#[main]
fn main(root: Handle) {
    println!("Hello, world!");

    let client = bindings::NameServer::new(Port::new(root));
    let res = client.Register("test str", 10).unwrap();

    println!("{:?}", res);
}

include!(concat!(env!("OUT_DIR"), "/hello.rs"));
