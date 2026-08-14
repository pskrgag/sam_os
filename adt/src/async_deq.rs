//! A vector whose asynchronous pop waits for an element.

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use rtl::locking::spinlock::Spinlock;

struct Waiter {
    id: usize,
    waker: Waker,
}

struct AsyncDeqInner<T> {
    values: VecDeque<T>,
    waiters: Vec<Waiter>,
    next_waiter_id: usize,
}

pub struct AsyncDeq<T> {
    inner: Spinlock<AsyncDeqInner<T>>,
}

impl<T> AsyncDeq<T> {
    pub const fn new() -> Self {
        Self {
            inner: Spinlock::new(AsyncDeqInner {
                values: VecDeque::new(),
                waiters: Vec::new(),
                next_waiter_id: 0,
            }),
        }
    }

    pub fn push_back(&self, value: T) {
        let waiter = {
            let mut inner = self.inner.lock();

            inner.values.push_back(value);
            inner.waiters.pop().map(|waiter| waiter.waker)
        };

        if let Some(waker) = waiter {
            waker.wake();
        }
    }

    pub fn pop_front(&self) -> Pop<'_, T> {
        Pop {
            vec: self,
            waiter_id: None,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.lock().values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().values.is_empty()
    }
}

impl<T> Default for AsyncDeq<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Pop<'a, T> {
    vec: &'a AsyncDeq<T>,
    waiter_id: Option<usize>,
}

impl<T> Future for Pop<'_, T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.vec.inner.lock();

        if let Some(value) = inner.values.pop_front() {
            if let Some(id) = self.waiter_id.take() {
                inner.waiters.retain(|waiter| waiter.id != id);
            }

            return Poll::Ready(value);
        }

        if let Some(id) = self.waiter_id {
            let waiter = inner
                .waiters
                .iter_mut()
                .find(|waiter| waiter.id == id)
                .expect("AsyncVec waiter is missing");

            if !waiter.waker.will_wake(cx.waker()) {
                waiter.waker = cx.waker().clone();
            }
        } else {
            let id = inner.next_waiter_id;
            inner.next_waiter_id = inner.next_waiter_id.wrapping_add(1);
            inner.waiters.push(Waiter {
                id,
                waker: cx.waker().clone(),
            });
            self.waiter_id = Some(id);
        }

        Poll::Pending
    }
}

impl<T> Drop for Pop<'_, T> {
    fn drop(&mut self) {
        if let Some(id) = self.waiter_id {
            self.vec
                .inner
                .lock()
                .waiters
                .retain(|waiter| waiter.id != id);
        }
    }
}
