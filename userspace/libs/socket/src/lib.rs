//! Socket API wrappers.
#![no_std]

extern crate alloc;

pub mod icmp;
mod socket;

pub use socket::{Socket, SocketProtocol};

use bindings_NetStack::NetStack as BindingNetStack;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;
use spin::Once;

static NETSTACK: Once<BindingNetStack> = Once::new();

/// # Safety
/// `handle` must refer to a NetStack service port.
pub unsafe fn init(handle: Handle) {
    NETSTACK.call_once(|| BindingNetStack::new(unsafe { Port::new(handle) }));
}

pub async fn socket<P: SocketProtocol>() -> Result<Socket<P>, ErrorType> {
    let response = NETSTACK.get().unwrap().Socket(P::PROTO).await?;

    Ok(unsafe { Socket::new(response.socket) })
}

include!(concat!(env!("OUT_DIR"), "/netstack.rs"));
