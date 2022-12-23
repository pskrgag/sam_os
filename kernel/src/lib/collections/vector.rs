use alloc::alloc::{alloc, dealloc, handle_alloc_error, realloc};
use core::{
    alloc::Layout,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

pub struct Vector<T: Sized> {
    size: usize,
    capacity: usize,
    data: NonNull<T>,
    _marker: PhantomData<T>,
}

pub struct IntoIter<T> {
    start: *const T,
    end: *const T,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for Vector<T> {}
unsafe impl<T: Sync> Sync for Vector<T> {}

impl<T: Sized> Vector<T> {
    pub const fn new() -> Self {
        Vector {
            data: NonNull::dangling(),
            size: 0,
            capacity: 0,
            _marker: PhantomData,
        }
    }

    pub fn with_capaicty(cap: usize) -> Option<Self> {
        let (cap, new_layout) = (cap, Layout::array::<T>(cap));

        if new_layout.is_err() {
            return None;
        }

        let layout = new_layout.unwrap();
        let array = unsafe { alloc(layout) };
        let ptr = NonNull::new(array as *mut T);

        if ptr.is_none() {
            return None;
        }

        Some(Vector {
            data: ptr.unwrap(),
            size: 0,
            capacity: cap,
            _marker: PhantomData,
        })
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.capacity == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.capacity;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            let old_ptr = self.data.as_ptr() as *mut u8;
            unsafe { realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.data = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => handle_alloc_error(new_layout),
        };
        self.capacity = new_cap;
    }

    pub fn push(&mut self, elem: T) {
        if self.size == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::write(self.data.as_ptr().add(self.size), elem);
        }

        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            unsafe { Some(ptr::read(self.data.as_ptr().add(self.size))) }
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            while let Some(_) = self.pop() {}
            let layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                dealloc(self.data.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T: Sized> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data.as_ptr(), self.size) }
    }
}

impl<T: Sized> Deref for Vector<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr(), self.size) }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / core::mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<'a, T> IntoIterator for &'a Vector<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        core::mem::forget(self);

        unsafe {
            IntoIter {
                start: self.data.as_ptr(),
                end: if self.capacity == 0 {
                    self.data.as_ptr()
                } else {
                    self.data.as_ptr().add(self.size)
                },
                _marker: PhantomData,
            }
        }
    }
}
