use core::sync::atomic::{AtomicUsize, Ordering, fence};
use core::ops::{Deref, DerefMut};

pub struct RefCounter(AtomicUsize);

pub struct Ref<'a, T> {
    pub(crate) data: &'a T,
    pub(crate) rc: &'a RefCounter,
}

pub struct RefMut<'a, T> {
    pub(crate) data: &'a mut T,
    pub(crate) rc: &'a RefCounter,
}

impl RefCounter {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(1))
    }

    pub fn acquire<T>(&self, data: &T) -> Ref<T> {
        self.0.fetch_add(1, Ordering::Acquire);

        Ref {
            data: data,
            rc: &self,
        }
    }

    pub fn acquire_mut<T>(&self, data: &mut T) -> RefMut<T> {
        self.0.fetch_add(1, Ordering::Acquire);

        RefMut {
            data: data,
            rc: &self,
        }
    }

    pub(crate) fn release(&self) -> bool {
        if self.0.fetch_sub(1, Ordering::Release) == 1 {
            fence(Ordering::Acquire);
            true
        } else {
            false
        }
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        if self.rc.release() {
            drop(*self.data);
        }
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.data
    }
}
