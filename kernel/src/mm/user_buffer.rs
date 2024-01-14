use alloc::boxed::Box;
use alloc::vec;
use rtl::vmm::types::*;

extern "C" {
    fn arch_copy_from_user(from: usize, size: usize, to: usize) -> isize;
    fn arch_copy_to_user(from: usize, size: usize, to: usize) -> isize;
}

#[derive(Debug)]
pub struct UserBuffer {
    va: VirtAddr,
    size: usize,
}

impl UserBuffer {
    pub fn new(va: VirtAddr, size: usize) -> Self {
        Self { va, size }
    }

    pub fn read_on_stack<const N: usize>(&self) -> Option<[u8; N]> {
        let mut arr = [0; N];

        let res =
            unsafe { arch_copy_from_user(self.va.into(), self.size, arr.as_mut_ptr() as usize) };
        if res == 0 {
            Some(arr)
        } else {
            None
        }
    }

    pub fn read_on_heap(&self, size: usize) -> Option<Box<[u8]>> {
        let mut arr = vec![0; size];

        let res =
            unsafe { arch_copy_from_user(self.va.into(), self.size, arr.as_mut_ptr() as usize) };
        if res == 0 {
            Some(arr.into_boxed_slice())
        } else {
            None
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Option<()> {
        println!("{} {}", data.len(), self.size);
        if data.len() <= self.size {
            let res =
                unsafe { arch_copy_to_user(data.as_ptr() as usize, data.len(), self.va.bits()) };
            if res == 0 {
                Some(())
            } else {
                None
            }
        } else {
            None
        }
    }
}
