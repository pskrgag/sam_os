use alloc::boxed::Box;
use alloc::vec;
use rtl::error::ErrorType;

extern "C" {
    fn arch_copy_from_user(from: usize, size: usize, to: usize) -> isize;
    fn arch_copy_to_user(from: usize, size: usize, to: usize) -> isize;
}

#[derive(Clone, Copy, Debug)]
pub struct UserPtr<T> {
    p: *const T,
    count: usize,
}

impl<T> UserPtr<T> {
    pub fn new(p: *const T) -> Self {
        Self { p, count: 1 }
    }

    pub fn new_array(p: *const T, count: usize) -> Self {
        Self { p, count }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn read_on_heap(&self) -> Option<Box<[u8]>> {
        use core::mem::size_of;

        let heap = vec![0; self.count * size_of::<T>()];

        unsafe {
            let res = arch_copy_from_user(
                self.p as usize,
                size_of::<T>() * self.count,
                heap.as_ptr() as _,
            );
            if res == 0 {
                Some(heap.into_boxed_slice())
            } else {
                None
            }
        }
    }

    pub fn read_to(&self, to: &mut [T]) -> Option<usize> {
        let s = usize::min(self.count, to.len());

        unsafe {
            let res = arch_copy_from_user(
                self.p as usize,
                core::mem::size_of::<T>() * s,
                to.as_ptr() as _,
            );

            if res == 0 {
                Some(s)
            } else {
                None
            }
        }
    }

    pub fn read(&self) -> Option<T> {
        use core::mem::{size_of, MaybeUninit};

        let t = MaybeUninit::uninit();

        unsafe {
            let res = arch_copy_from_user(
                self.p as usize,
                size_of::<T>() * self.count,
                t.as_ptr() as _,
            );
            if res == 0 {
                Some(t.assume_init())
            } else {
                println!("{:?}", self.p);
                panic!("");
                None
            }
        }
    }

    pub fn write(mut self, t: &T) -> Result<(), ErrorType> {
        use core::mem::size_of;

        unsafe {
            let res = arch_copy_to_user(
                t as *const _ as usize,
                size_of::<T>() * self.count,
                self.p as usize,
            );
            if res == 0 {
                Ok(())
            } else {
                Err(ErrorType::FAULT)
            }
        }
    }

    pub fn write_array(&mut self, t: &[T]) -> Result<(), ErrorType> {
        use core::mem::size_of;

        let s = usize::min(self.count, t.len());

        unsafe {
            let res = arch_copy_to_user(t.as_ptr() as usize, size_of::<T>() * s, self.p as usize);
            if res == 0 {
                Ok(())
            } else {
                Err(ErrorType::FAULT)
            }
        }
    }
}
