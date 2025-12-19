use crate::arch::irq::interrupts::{IrqFlags, get_flags, set_flags};
use crate::kernel::object::thread_object::Thread;
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicPtr, AtomicU16, Ordering},
};

#[derive(Debug)]
pub struct SpinLockInner {
    current: AtomicU16,
    next: AtomicU16,
    t: AtomicPtr<Thread>,
}

#[derive(Debug)]
pub struct Spinlock<T> {
    inner: SpinLockInner,
    val: UnsafeCell<T>,
}

pub struct SpinlockGuard<'a, T: 'a> {
    lock: &'a SpinLockInner,
    data: &'a mut T,
    flags: Option<IrqFlags>,
}

impl<'a, T> SpinlockGuard<'a, T> {
    pub unsafe fn into<U>(self, data: &'a mut U) -> SpinlockGuard<'a, U> {
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
                t: AtomicPtr::new(core::ptr::null_mut()),
            },
            val: UnsafeCell::new(val),
        }
    }

    pub fn lock<'a>(&'a self) -> SpinlockGuard<'a, T> {
        use crate::kernel::sched::current::get_current_raw;

        if let Some(cur) = get_current_raw()
            && cur == self.inner.t.load(Ordering::Relaxed)
        {
            panic!("Deadlock");
        }

        let my = self.inner.next.fetch_add(1, Ordering::Acquire);

        while self.inner.current.load(Ordering::Relaxed) != my {
            core::hint::spin_loop();
        }

        if let Some(cur) = get_current_raw() {
            self.inner.t.store(cur as *mut _, Ordering::Relaxed);
        }

        SpinlockGuard {
            lock: &self.inner,
            data: unsafe { &mut *self.val.get() },
            flags: None,
        }
    }

    pub fn lock_irqsave<'a>(&'a self) -> SpinlockGuard<'a, T> {
        let my = self.inner.next.fetch_add(1, Ordering::Acquire);

        while self.inner.current.load(Ordering::Relaxed) != my {
            core::hint::spin_loop();
        }

        let flags = Some(get_flags());
        arm_gic::irq_disable();

        SpinlockGuard {
            lock: &self.inner,
            data: unsafe { &mut *self.val.get() },
            flags,
        }
    }
}

impl SpinLockInner {
    pub fn unlock(&self) {
        self.t.store(core::ptr::null_mut(), Ordering::Relaxed);

        self.current.fetch_add(1, Ordering::Release);
    }

    pub fn unlock_irqrestore(&self, flags: IrqFlags) {
        unsafe {
            set_flags(flags);
        }

        self.unlock();
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
        if let Some(f) = self.flags {
            self.lock.unlock_irqrestore(f);
        } else {
            self.lock.unlock();
        }
    }
}

unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}
