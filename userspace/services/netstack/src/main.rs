#![no_std]
#![no_main]

use bindings_NameServer::NameServer;
use libc::handle::Handle;
use net::eth::{
    arp::{Arp, ArpOperation},
    frame::Frame,
};
use rokio::port::Port;
use rtl::error::ErrorType;

mod nic;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let ns = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let nic = nic::Nic::new(&ns).await?;

    let mac = nic.mac().await?;

    println!("Hello, net!");

    loop {
        let packet = nic.read_packet().await?;
        let frame: Frame = packet.as_slice().try_into().unwrap();
        let arp = frame.payload::<Arp>()?;

        println!("received shit {:?}", arp);

        let arp_reply = Arp::new(
            ArpOperation::Reply,
            mac,
            [192, 168, 100, 2],
            arp.sender_mac(),
            arp.sender_ip(),
        );

        let reply_frame = Frame::serialize(frame.source(), mac, arp_reply);
        nic.send_packet(&reply_frame).await?;
    }
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
