use super::executor::Waiter;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use libc::port::Port as LibcPort;
use rtl::error::ErrorType;
use rtl::ipc::IpcMessage;

pub struct Port {
    port: LibcPort,
}

impl Port {
    pub fn create() -> Result<Self, ErrorType> {
        LibcPort::create().map(|port| Self { port })
    }

    pub async fn call<'a>(&'a self, msg: &'a mut IpcMessage<'a>) -> Result<usize, ErrorType> {
        let reply_port = self.port.send(msg)?;

        struct ReplyFuture<'a> {
            port: &'a Port,
            msg: &'a mut IpcMessage<'a>,
        }

        impl Future for ReplyFuture<'_> {
            type Output = Result<usize, ErrorType>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let cur = self.get_mut();

                match cur.port.port.receive(cur.msg) {
                    Ok(received) => Poll::Ready(Ok(received)),
                    Err(ErrorType::WouldBlock) => {
                        let waiter = Waiter::new(
                            unsafe { cur.port.port.handle().as_raw() },
                            rtl::signal::Signal::MessageReady.into(),
                            cx.waker().clone(),
                        );

                        super::executor::current_runtime().add_wait(waiter);
                        Poll::Pending
                    }
                    Err(err) => Poll::Ready(Err(err)),
                }
            }
        }

        ReplyFuture { port: self, msg }.await
    }
}
