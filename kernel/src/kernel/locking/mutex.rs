use super::spinlock::Spinlock;
use crate::kernel::object::thread_object::Thread;
use crate::kernel::sched::current;
use alloc::collections::LinkedList;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU8, Ordering};
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

const FREE: u8 = 0;
const LOCKED: u8 = 1;

pub struct MutexGuard<'a, T> {
    mtx: &'a Mutex<T>,
}

pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    state: AtomicU8,
    waiters: Spinlock<LinkedList<Arc<Thread>>>,
}

impl<T> Mutex<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            state: AtomicU8::new(FREE),
            waiters: Spinlock::new(LinkedList::new()),
        }
    }

    fn lock_slow<'a>(&'a self) -> MutexGuard<'a, T> {
        let cur = current();
        let mut list = self.waiters.lock();

        if self
            .state
            .compare_exchange(FREE, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return MutexGuard { mtx: self };
        }

        list.push_back(cur.clone());
        // cur.sleep(ThreadSleepReason::Mutex);

        debug_assert!(self.state.load(Ordering::Relaxed) != FREE);
        MutexGuard { mtx: self }
    }

    fn unlock(&self) {
        let mut list = self.waiters.lock();

        if let Some(waiter) = list.pop_front() {
            waiter.wake();
        } else {
            self.state.store(FREE, Ordering::Release);
        }
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        match self
            .state
            .compare_exchange(FREE, LOCKED, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => MutexGuard { mtx: self },
            Err(_) => self.lock_slow(),
        }
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

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
