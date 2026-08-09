use super::NetStack as NetworkStack;
use crate::bindings_NameServer::NameServer;
use crate::socket::{Socket, server};
use alloc::sync::Arc;
use bindings_NetStack::{NetStack, NetStackRequest};
pub(crate) use bindings_NetStack::{Socket as SocketBindings, SocketRequest};
use rokio::port::Port;
use rtl::error::ErrorType;

pub async fn serve(netstack: Arc<NetworkStack>, ns: NameServer) -> Result<(), ErrorType> {
    let port = Port::create()?;

    ns.Register("netstack".try_into().unwrap(), port.handle())
        .await
        .expect("Failed to register handle in nameserver");

    NetStack::for_each(port, move |req| {
        let netstack = netstack.clone();

        async move {
            match req {
                NetStackRequest::Socket { responder, .. } => {
                    let sock = Socket::new(super::inet::icmp::IcmpSocket, netstack);

                    let (handler, handle) = server::new(sock)?;

                    rokio::executor::spawn(handler);
                    responder.reply(&handle)?;
                }
            }

            Ok(())
        }
    })
    .await
}

include!(concat!(env!("OUT_DIR"), "/netstack.rs"));
