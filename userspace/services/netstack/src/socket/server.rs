use super::{Socket as NetSocket, SocketOps};
use crate::netstack::server::{SocketBindings, SocketRequest};
use alloc::sync::Arc;
use core::future::Future;
use heapless::Vec;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;
use crate::netstack::server::Address;

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
                        socket
                            .send_to(value.address.ipv4.into(), &value.data[..value.size])
                            .await?;
                        responder.reply(0)
                    }
                    SocketRequest::Receive { value, responder } => {
                        let mut data = Vec::new();

                        data.resize(1024, 0).unwrap();
                        let (size, address) = socket.recv_from(&mut data[..value.size]).await?;

                        responder.reply(
                            data,
                            size,
                            Address {
                                ipv4: u32::from_ne_bytes(address.as_slice().try_into().unwrap()),
                            },
                        )
                    }
                }
            }
        }),
        raw_handle,
    ))
}
