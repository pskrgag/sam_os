use super::spinlock::Spinlock;
use adt::Vec;
use core::future::Future;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::{Context, Poll, Waker};
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};
use rtl::error::ErrorType;

const FREE: u8 = 0;
const LOCKED: u8 = 1;

pub struct MutexGuard<'a, T> {
    mtx: &'a Mutex<T>,
}

pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    state: AtomicU8,
    waiters: Spinlock<Vec<Waker>>,
}

impl<T> Mutex<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            state: AtomicU8::new(FREE),
            waiters: Spinlock::new(Vec::new()),
        }
    }

    fn unlock(&self) {
        let mut waiters = self.waiters.lock();

        self.state.store(FREE, Ordering::Release);
        if let Some(waiter) = waiters.pop() {
            waiter.wake();
        }
    }

    pub fn try_lock<'a>(&'a self) -> Option<MutexGuard<'a, T>> {
        match self
            .state
            .compare_exchange(FREE, LOCKED, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => Some(MutexGuard { mtx: self }),
            Err(_) => None,
        }
    }

    pub async fn lock<'a>(&'a self) -> Result<MutexGuard<'a, T>, ErrorType> {
        struct MutexFuture<'a, T> {
            mutex: &'a Mutex<T>,
        }

        impl<'a, T> Future for MutexFuture<'a, T> {
            type Output = Result<MutexGuard<'a, T>, ErrorType>;

            fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut waiters = self.mutex.waiters.lock();

                if let Some(guard) = self.mutex.try_lock() {
                    Poll::Ready(Ok(guard))
                } else {
                    waiters.try_push(cx.waker().clone())?;
                    Poll::Pending
                }
            }
        }

        MutexFuture { mutex: self }.await
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mtx.inner.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mtx.inner.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mtx.unlock();
    }
}
