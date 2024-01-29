use core::cell::UnsafeCell;

pub struct FakeLock<T> {
    val: UnsafeCell<T>,
}

impl<T> FakeLock<T> {
    pub const fn new(val: T) -> Self {
        Self {
            val: UnsafeCell::new(val),
        }
    }

    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.val.get() }
    }
}

unsafe impl<T> Sync for FakeLock<T> {}
