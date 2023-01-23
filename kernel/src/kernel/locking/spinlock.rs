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
    flags: Option<usize>,
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
        let my = self.next.fetch_add(1, Ordering::Acquire);

        while self.current.load(Ordering::Relaxed) != my {
            unsafe { asm!("wfe") };
        }

        SpinlockGuard {
            lock: &self,
            data: unsafe { &mut *self.val.get() },
            flags: None,
        }
    }

    pub fn lock_irqsave(&self) -> SpinlockGuard<T> {
        use crate::arch::irq::{disable_all, get_flags};

        let flags = Some(get_flags());

        unsafe {
            disable_all();
        }

        let my = self.next.fetch_add(1, Ordering::Acquire);

        while self.current.load(Ordering::Relaxed) != my {
            unsafe { asm!("wfe") };
        }

        SpinlockGuard {
            lock: &self,
            data: unsafe { &mut *self.val.get() },
            flags: flags,
        }
    }

    pub fn unlock(&self) {
        self.current.fetch_add(1, Ordering::Release);
    }

    pub fn unlock_irqrestore(&self, flags: usize) {
        use crate::arch::irq::set_flags;

        self.unlock();

        unsafe {
            set_flags(flags);
        }
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
