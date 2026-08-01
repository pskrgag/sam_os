use super::bindings_NameServer::NameServer;
use crate::driver::Nic;
use alloc::boxed::Box;
use alloc::sync::Arc;
use bindings_Nic::{Nic as NicBindings, NicRequest};
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub async fn start_server(nic: Box<dyn Nic>, ns: NameServer) -> Result<(), ErrorType> {
    let port = Port::create()?;
    let nic = Arc::new(Spinlock::new(nic));

    ns.Register("nic".try_into().unwrap(), port.handle())
        .await
        .expect("Failed to register handle in nameserver");

    NicBindings::for_each(port, move |req| {
        let nic = nic.clone();

        async move {
            match req {
                NicRequest::Receive { responder, .. } => {
                    let mut nic = nic.lock();
                    let data = nic.receive_frame()?;

                    responder.reply(data.into_iter().collect())?;
                }
                NicRequest::Send { value, responder } => {
                    let mut nic = nic.lock();

                    nic.send_frame(&value.data)?;
                    responder.reply()?;
                }
                NicRequest::Mac { responder, .. } => {
                    let nic = nic.lock();
                    let mac = nic.mac();

                    responder.reply(mac.into())?;
                }
            }
            Ok(())
        }
    })
    .await
}

include!(concat!(env!("OUT_DIR"), "/nic.rs"));
