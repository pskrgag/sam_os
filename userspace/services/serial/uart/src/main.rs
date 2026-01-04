#![no_main]
#![no_std]

use alloc::sync::Arc;
use bindings_Serial::{GetByteRx, PutRx, Serial, SerialRequest};
use fdt::Fdt;
use hal::address::VirtualAddress;
use libc::handle::Handle;
use libc::syscalls::Syscall;
use rokio::port::Port;
use rtl::locking::spinlock::Spinlock;

mod pl011;

#[rokio::main]
async fn main(nameserver: Option<Handle>) {
    let fdt = Syscall::get_fdt().unwrap();
    let fdt = unsafe { Fdt::from_ptr(fdt.to_raw::<u8>()).unwrap() };

    let pl011 = Arc::new(Spinlock::new(pl011::probe(&fdt).unwrap()));
    let p = Port::create().unwrap();

    let nameserver =
        bindings_NameServer::NameServer::new(unsafe { Port::new(nameserver.unwrap()) });

    nameserver
        .Register("serial".try_into().unwrap(), p.handle().clone())
        .await
        .expect("Failed to register handle in nameserver");

    println!("Starting 'uart' server...");

    Serial::for_each(p, |req| {
        let pl011 = pl011.clone();

        async move {
            match req {
                SerialRequest::Put { value, responder } => {
                    let mut pl011 = pl011.lock();

                    for bt in value.message.bytes() {
                        pl011.write_byte(bt);
                    }

                    responder.reply(PutRx {})?;
                }
                SerialRequest::GetByte { responder, .. } => {
                    let byte = pl011.lock().read_byte();

                    responder.reply(GetByteRx { byte })?;
                }
            };

            Ok(())
        }
    })
    .await
    .unwrap();
}

include!(concat!(env!("OUT_DIR"), "/serial.rs"));
include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
