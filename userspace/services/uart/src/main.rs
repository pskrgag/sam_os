#![no_main]
#![no_std]

use fdt::Fdt;
use hal::address::VirtualAddress;
use libc::syscalls::Syscall;
use libc::{handle::Handle, main, port::Port};
use rtl::locking::spinlock::Spinlock;

mod pl011;

#[main]
fn main(nameserver: Handle) {
    let fdt = Syscall::get_fdt().unwrap();
    let fdt = unsafe { Fdt::from_ptr(fdt.to_raw::<u8>()).unwrap() };

    let pl011 = pl011::probe(&fdt).unwrap();
    let p = Port::create().unwrap();

    let nameserver = bindings_NameServer::NameServer::new(Port::new(nameserver));
    nameserver
        .Register("serial", p.handle())
        .expect("Failed to register handle in nameserver");

    let mut server = bindings_Serial::Serial::new(p, Spinlock::new(pl011))
        .register_handler(|_: bindings_Serial::GetByteTx, uart| {
            Ok(bindings_Serial::GetByteRx {
                byte: uart.lock().read_byte(),
            })
        })
        .register_handler(|msg: bindings_Serial::PutTx, uart| {
            let mut uart = uart.lock();

            for i in msg.message.bytes() {
                uart.write_byte(i);
            }
            Ok(bindings_Serial::PutRx {})
        });

    println!("Starting 'uart' server...");
    server.run().unwrap();
}

include!(concat!(env!("OUT_DIR"), "/serial.rs"));
include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
