use super::executor::Waiter;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use libc::handle::Handle;
use libc::port::Port as LibcPort;
use rtl::error::ErrorType;
use rtl::ipc::IpcMessage;

pub struct Port {
    port: LibcPort,
}

struct RecvFuture<'a> {
    port: &'a LibcPort,
    msg: usize,
}

impl Future for RecvFuture<'_> {
    type Output = Result<usize, ErrorType>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let cur = self.get_mut();
        let msg = unsafe { &mut *(cur.msg as *mut IpcMessage) };

        let res = match cur.port.receive(msg) {
            Ok(received) => Poll::Ready(Ok(received)),
            Err(ErrorType::WouldBlock) => {
                let waiter = Waiter::new(
                    unsafe { cur.port.handle().as_raw() },
                    rtl::signal::Signal::MessageReady.into(),
                    cx.waker().clone(),
                );

                super::executor::current_runtime().add_wait(waiter);
                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(err)),
        };

        res
    }
}

impl Port {
    pub fn create() -> Result<Self, ErrorType> {
        LibcPort::create().map(|port| Self { port })
    }

    pub unsafe fn new(h: Handle) -> Self {
        Self {
            port: unsafe { LibcPort::new(h) },
        }
    }

    pub fn handle(&self) -> &Handle {
        self.port.handle()
    }

    pub async fn call(&self, msg: &mut IpcMessage<'_>) -> Result<usize, ErrorType> {
        let reply_port = self.port.send(msg)?;

        RecvFuture {
            port: &reply_port,
            msg: msg as *mut _ as usize,
        }
        .await
    }

    pub fn reply(&self, reply_port: Handle, msg: &IpcMessage) -> Result<(), ErrorType> {
        self.port.reply(reply_port, msg)
    }

    pub async fn receive(&self, msg: &mut IpcMessage<'_>) -> Result<usize, ErrorType> {
        RecvFuture {
            port: &self.port,
            msg: msg as *mut _ as usize,
        }
        .await
    }
}
