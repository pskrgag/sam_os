use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use core::ops::{Deref, DerefMut};
use rtl::error::ErrorType;

pub struct Vec<T>(alloc::vec::Vec<T>);

impl<T> Deref for Vec<T> {
    type Target = alloc::vec::Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Vec<T> {
    pub fn try_push(&mut self, val: T) -> Result<(), ErrorType> {
        match self.0.push_within_capacity(val) {
            Ok(_) => Ok(()),
            Err(val) => {
                self.0
                    .try_reserve(self.0.capacity().max(8))
                    .map_err(|_| ErrorType::NoMemory)?;

                let res = self.0.push_within_capacity(val).is_ok();
                debug_assert!(res);
                Ok(())
            }
        }
    }

    pub fn new() -> Self {
        Self(alloc::vec::Vec::new())
    }

    pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Self {
        unsafe { Self(alloc::vec::Vec::from_raw_parts(ptr, length, capacity)) }
    }

    pub fn into_boxed_slice(self) -> Box<[T]> {
        self.0.into_boxed_slice()
    }

    #[deprecated(note = "Please use `try_push` instead")]
    pub fn push(&mut self, _val: T) {
        unreachable!()
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self(alloc::vec::Vec::new())
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T: Debug> Debug for Vec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
