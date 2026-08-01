use super::spinlock::Spinlock;
use adt::Vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, Waker};
use rtl::error::ErrorType;

#[repr(u8)]
#[derive(PartialEq)]
enum EventState {
    NotSignaled = 0,
    Signaled = 1,
}

struct Waiter {
    notified: AtomicBool,
    waker: Spinlock<Option<Waker>>,
}

struct EventInner {
    state: EventState,
    waiters: Vec<Arc<Waiter>>,
}

pub struct Event {
    inner: Spinlock<EventInner>,
}

impl Event {
    pub fn new() -> Self {
        let inner = EventInner {
            state: EventState::NotSignaled,
            waiters: Vec::new(),
        };

        Self {
            inner: Spinlock::new(inner),
        }
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.state = EventState::NotSignaled;
    }

    pub fn broadcast(&self) {
        let waiters = {
            let mut inner = self.inner.lock();
            let items = core::mem::replace(&mut inner.waiters, Vec::new());

            inner.state = EventState::Signaled;
            items.into_iter().inspect(|x| {
                x.notified.store(true, Ordering::Release);
            })
        };

        for waiter in waiters {
            let mut waker = waiter.waker.lock();

            if let Some(waker) = waker.take() {
                waker.wake();
            }
        }
    }

    pub async fn wait(&self) -> Result<(), ErrorType> {
        struct WaitFuture<'a> {
            event: &'a Event,
            waiter: Arc<Waiter>,
            polled: bool,
        }

        impl Future for WaitFuture<'_> {
            type Output = Result<(), ErrorType>;

            fn poll(
                mut self: core::pin::Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Self::Output> {
                if self.waiter.notified.load(Ordering::Acquire) {
                    return Poll::Ready(Ok(()));
                }

                let mut inner = self.event.inner.lock();

                if inner.state == EventState::Signaled {
                    return Poll::Ready(Ok(()));
                }

                *self.waiter.waker.lock() = Some(cx.waker().clone());

                if !self.polled {
                    inner.waiters.try_push(self.waiter.clone())?;
                    self.polled = true;
                }

                Poll::Pending
            }
        }

        WaitFuture {
            event: self,
            polled: false,
            waiter: Arc::try_new(Waiter {
                notified: AtomicBool::new(false),
                waker: Spinlock::new(None),
            })
            .map_err(|_| ErrorType::NoMemory)?,
        }
        .await
    }
}
