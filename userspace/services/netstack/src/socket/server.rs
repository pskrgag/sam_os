use super::{Socket as NetSocket, SocketOps};
use crate::netstack::server::{SocketBindings, SocketRequest};
use alloc::sync::Arc;
use core::future::Future;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

fn pending<T>() -> T {
    todo!()
}

pub fn new<S: SocketOps + Sync + 'static>(
    socket: Arc<NetSocket<S>>,
) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
    let port = Port::create()?;
    let raw_handle = port.handle().clone_handle()?;

    Ok((
        SocketBindings::for_each(port, move |req| {
            let socket = socket.clone();

            async move {
                match req {
                    SocketRequest::SendTo { value, responder } => {
                        let _ = (socket, value, responder);
                        pending()
                    }
                    SocketRequest::Receive { value, responder } => {
                        let _ = (socket, value, responder);
                        pending()
                    }
                }
            }
        }),
        raw_handle,
    ))
}
