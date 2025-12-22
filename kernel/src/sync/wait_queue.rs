use super::mutex::Mutex;
use crate::adt::Vec;
use alloc::collections::VecDeque;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use rtl::error::ErrorType;

pub struct WaitQueue<T> {
    data: Mutex<VecDeque<T>>,
    waiters: Mutex<Vec<Waker>>,
}

impl<T> WaitQueue<T> {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(VecDeque::new()),
            waiters: Mutex::new(Vec::new()),
        }
    }

    pub fn produce(&self, data: T) {
        self.data.lock().push_back(data);

        if let Some(waiter) = self.waiters.lock().pop() {
            waiter.wake();
        }
    }

    pub async fn consume(&self) -> Result<T, ErrorType> {
        struct ConsumeFuture<'a, T> {
            wq: &'a WaitQueue<T>,
        }

        impl<'a, T> Future for ConsumeFuture<'a, T> {
            type Output = Result<T, ErrorType>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut data = self.wq.data.lock();

                if let Some(elem) = data.pop_front() {
                    Poll::Ready(Ok(elem))
                } else {
                    self.wq.waiters.lock().try_push(cx.waker().clone())?;
                    Poll::Pending
                }
            }
        }

        ConsumeFuture { wq: self }.await
    }
}
