use core::{
    arch::asm,
    cell::UnsafeCell,
    ops::{Deref, DerefMut, Drop},
    sync::atomic::{AtomicU16, Ordering},
};

pub struct Spinlock<T> {
    current: AtomicU16,
    next: AtomicU16,
    val: UnsafeCell<T>,
}

pub struct SpinlockGuard<'a, T: 'a> {
    lock: &'a Spinlock<T>,
    data: &'a mut T,
}

impl<T> Spinlock<T> {
    pub const fn new(val: T) -> Self {
        Self {
            current: AtomicU16::new(0),
            next: AtomicU16::new(0),
            val: UnsafeCell::new(val),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<T> {
        let my = self.current.fetch_add(1, Ordering::Acquire);

        while self.current.load(Ordering::Relaxed) != my {
            unsafe { asm!("yield") };
        }

        SpinlockGuard {
            lock: &self,
            data: unsafe { &mut *self.val.get() },
        }
    }

    pub fn unlock(&self) {
        self.current.fetch_add(1, Ordering::Relaxed);
    }
}

impl<'a, T> Deref for SpinlockGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &*self.data
    }
}

impl<'a, T> DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
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

