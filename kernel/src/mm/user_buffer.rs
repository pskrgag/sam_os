use core::mem::{size_of, MaybeUninit};
use rtl::vmm::types::VirtAddr;

extern "C" {
    fn arch_copy_from_user(from: usize, size: usize, to: usize) -> i32;
}

pub struct UserBuffer<const N: usize> {
    data: [u8; N],
    size: usize,
}

impl<const N: usize> UserBuffer<N> {
    pub fn new(v: VirtAddr, size: usize) -> Option<Self> {
        let mut s = MaybeUninit::<Self>::uninit();
        let p = s.as_mut_ptr();

        let res = unsafe { arch_copy_from_user(v.into(), size, (*p).data.as_mut_ptr() as usize) };
        if res == 0 {
            unsafe { (*p).size = size };
            Some(unsafe { s.assume_init() })
        } else {
            None
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.size]
    }
}
