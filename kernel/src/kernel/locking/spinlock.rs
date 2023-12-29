use core::{
    arch::asm,
    cell::UnsafeCell,
    ops::{Deref, DerefMut, Drop},
    sync::atomic::{AtomicU16, Ordering},
};

pub struct SpinLockInner {
    current: AtomicU16,
    next: AtomicU16,
}

pub struct Spinlock<T> {
    inner: SpinLockInner,
    val: UnsafeCell<T>,
}

pub struct SpinlockGuard<'a, T: 'a> {
    lock: &'a SpinLockInner,
    data: &'a mut T,
    flags: Option<usize>,
}

impl<'a, T> SpinlockGuard<'a, T> {
    pub unsafe fn into<U>(self: Self, data: &'a mut U) -> SpinlockGuard<U> {
        let res = SpinlockGuard {
            lock: self.lock,
            data,
            flags: self.flags,
        };

        core::mem::forget(self);
        res
    }

    pub unsafe fn force_unlock(&mut self) {
        self.lock.unlock();
    }
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
            unsafe { asm!("yield") };
        }

        SpinlockGuard {
            lock: &self.inner,
            data: unsafe { &mut *self.val.get() },
            flags: None,
        }
    }

    pub fn lock_irqsave(&self) -> SpinlockGuard<T> {
        use crate::arch::irq::{disable_all, get_flags};
        let my = self.inner.next.fetch_add(1, Ordering::Acquire);

        while self.inner.current.load(Ordering::Relaxed) != my {
            unsafe { asm!("yield") };
        }

        let flags = Some(get_flags());

        unsafe {
            disable_all();
        }

        SpinlockGuard {
            lock: &self.inner,
            data: unsafe { &mut *self.val.get() },
            flags,
        }
    }
}

impl SpinLockInner {
    pub fn unlock(&self) {
        self.current.fetch_add(1, Ordering::Release);
    }

    pub fn unlock_irqrestore(&self, flags: usize) {
        use crate::arch::irq::set_flags;

        unsafe {
            set_flags(flags);
        }

        self.unlock();
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
        if let Some(f) = self.flags {
            self.lock.unlock_irqrestore(f);
        } else {
            self.lock.unlock();
        }
    }
}

unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}
