use super::executor::{Waiter, WaiterState};
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use libc::handle::Handle;
use libc::irq::Irq as LibcIrq;
use rtl::error::ErrorType;
use rtl::irq::IrqTrigger;

pub struct Irq {
    irq: LibcIrq,
}

struct IrqFuture<'a> {
    irq: &'a Irq,
    state: Option<Arc<WaiterState>>,
}

impl Future for IrqFuture<'_> {
    type Output = Result<(), ErrorType>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let cur = self.get_mut();

        if let Some(ref state) = cur.state {
            if state.completed() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        } else {
            let state = WaiterState::new(cx.waker().clone());

            let waiter = Waiter::new(
                unsafe { cur.irq.irq.handle().as_raw() },
                rtl::signal::Signal::IrqReady.into(),
                state.clone(),
            );

            cur.state = Some(state);
            super::executor::current_runtime().add_wait(waiter);
            Poll::Pending
        }
    }
}

impl Irq {
    pub unsafe fn new_from_handle(h: Handle) -> Self {
        unsafe {
            Self {
                irq: LibcIrq::new(h),
            }
        }
    }

    pub fn new(num: usize, trigger: IrqTrigger) -> Result<Self, ErrorType> {
        let irq = LibcIrq::create(num, trigger)?;

        Ok(Self { irq })
    }

    pub fn ack(&self) {
        self.irq.ack().unwrap();
    }

    pub async fn wait(&self) -> Result<(), ErrorType> {
        IrqFuture {
            irq: self,
            state: None,
        }
        .await
    }
}
