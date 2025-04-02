use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU16, Ordering},
};

#[derive(Debug)]
pub struct SpinLockInner {
    current: AtomicU16,
    next: AtomicU16,
}

#[derive(Debug)]
pub struct Spinlock<T> {
    inner: SpinLockInner,
    val: UnsafeCell<T>,
}

pub struct SpinlockGuard<'a, T: 'a> {
    lock: &'a SpinLockInner,
    data: &'a mut T,
}

impl<T> Spinlock<T> {
    pub const fn new(val: T) -> Self {
        Self {
            inner: SpinLockInner {
                current: AtomicU16::new(0),
                next: AtomicU16::new(0),
            },
            val: UnsafeCell::new(val),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<T> {
        let my = self.inner.next.fetch_add(1, Ordering::Acquire);

        while self.inner.current.load(Ordering::Relaxed) != my {
            core::hint::spin_loop();
        }

        SpinlockGuard {
            lock: &self.inner,
            data: unsafe { &mut *self.val.get() },
        }
    }

    pub fn lock_irqsave(&self) -> SpinlockGuard<T> {
        let my = self.inner.next.fetch_add(1, Ordering::Acquire);

        while self.inner.current.load(Ordering::Relaxed) != my {
            core::hint::spin_loop();
        }

        SpinlockGuard {
            lock: &self.inner,
            data: unsafe { &mut *self.val.get() },
        }
    }
}

impl SpinLockInner {
    pub fn unlock(&self) {
        self.current.fetch_add(1, Ordering::Release);
    }
}

impl<'a, T> Deref for SpinlockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T> DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}
