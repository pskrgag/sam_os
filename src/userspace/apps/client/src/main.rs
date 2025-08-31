#![no_main]
#![no_std]

use libc::handle::Handle;
use libc::{main, port::Port};

#[main]
fn main(root: Handle) {
    let server = Port::create().unwrap();
    println!("Hello, world! {:x}", unsafe {
        (0x21d904 as *const u8).read_volatile()
    });

    let client = bindings::NameServer::new(Port::new(root));
    let res = client.Register("test str", server.handle()).unwrap();
    let res = client.Get("test str").unwrap();

    println!("{:?}", res);
}

include!(concat!(env!("OUT_DIR"), "/hello.rs"));
