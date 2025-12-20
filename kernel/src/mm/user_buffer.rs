use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use rtl::error::ErrorType;

unsafe extern "C" {
    fn arch_copy_from_user(from: usize, size: usize, to: usize) -> isize;
    fn arch_copy_to_user(from: usize, size: usize, to: usize) -> isize;
}

#[derive(Clone, Copy, Debug)]
pub struct UserPtr<T> {
    p: usize,
    count: usize,
    _p: PhantomData<T>,
}

// TODO: this API is fucking garbage and needs redo
impl<T> UserPtr<T> {
    pub fn new(p: *const T) -> Self {
        Self {
            p: p as usize,
            count: 1,
            _p: PhantomData,
        }
    }

    pub fn new_array(p: *const T, count: usize) -> Self {
        Self {
            p: p as usize,
            count,
            _p: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn read_on_heap(&self) -> Result<Box<[T]>, ErrorType> {
        use core::mem::size_of;

        // TODO: add error handling of OMM. This is no good
        // let heap = vec![0; self.count * size_of::<T>()];
        let mut heap = Vec::new();
        let len = self.count * size_of::<T>();

        heap.try_reserve_exact(len)
            .map_err(|_| ErrorType::NoMemory)?;
        let slice = heap.spare_capacity_mut();

        unsafe {
            let res = arch_copy_from_user(
                self.p as usize,
                size_of::<T>() * self.count,
                slice.as_ptr() as _,
            );
            if res == 0 {
                heap.set_len(self.count);
                Ok(heap.into_boxed_slice())
            } else {
                Err(ErrorType::Fault)
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
                None
            }
        }
    }

    pub fn write(&mut self, t: &T) -> Result<(), ErrorType> {
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
                Err(ErrorType::Fault)
            }
        }
    }

    pub fn write_array(&mut self, t: &[T]) -> Result<(), ErrorType> {
        use core::mem::size_of;

        if self.count < t.len() {
            return Err(ErrorType::InvalidArgument);
        }

        unsafe {
            let res = arch_copy_to_user(
                t.as_ptr() as usize,
                size_of::<T>() * t.len(),
                self.p as usize,
            );
            if res == 0 {
                Ok(())
            } else {
                Err(ErrorType::Fault)
            }
        }
    }
}
