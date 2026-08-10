use super::executor::{Waiter, WaiterState};
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use libc::timer::Timer as LibcTimer;
use rtl::error::ErrorType;

pub struct Timer;

struct TimerFuture<'a> {
    timer: &'a LibcTimer,
    dl: Duration,
    state: Option<Arc<WaiterState>>,
}

impl Future for TimerFuture<'_> {
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

            cur.timer.arm(cur.dl)?;

            let waiter = Waiter::new(
                unsafe { cur.timer.handle().as_raw() },
                rtl::signal::Signal::TimerReady.into(),
                state.clone(),
            );

            cur.state = Some(state);
            super::executor::current_runtime().add_wait(waiter);
            Poll::Pending
        }
    }
}

impl Timer {
    pub async fn wait(dl: Duration) -> Result<(), ErrorType> {
        let timer = LibcTimer::create()?;

        TimerFuture {
            timer: &timer,
            state: None,
            dl,
        }
        .await
    }
}
