use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct FakeLock<T> {
    val: UnsafeCell<T>,
}

pub struct Wrapper<'a, T>(&'a mut T);

impl<T> FakeLock<T> {
    pub const fn new(val: T) -> Self {
        Self {
            val: UnsafeCell::new(val),
        }
    }

    pub fn get<'a>(&'a self) -> Wrapper<'a, T> {
        Wrapper(unsafe { &mut *self.val.get() })
    }
}

impl<'a, T> Deref for Wrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.0
    }
}

impl<'a, T> DerefMut for Wrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.0
    }
}

unsafe impl<T> Sync for FakeLock<T> {}
