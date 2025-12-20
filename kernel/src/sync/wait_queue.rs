use super::mutex::Mutex;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

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

    pub async fn consume(&self) -> T {
        struct ConsumeFuture<'a, T> {
            wq: &'a WaitQueue<T>,
        }

        impl<'a, T> Future for ConsumeFuture<'a, T> {
            type Output = T;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut data = self.wq.data.lock();

                if let Some(elem) = data.pop_front() {
                    Poll::Ready(elem)
                } else {
                    self.wq.waiters.lock().push(cx.waker().clone());
                    Poll::Pending
                }
            }
        }

        ConsumeFuture { wq: self }.await
    }
}
