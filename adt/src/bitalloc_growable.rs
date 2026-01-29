//! Bitmap allocator

use super::bitalloc::BitAllocator;

pub struct GrowBitAllocator {
    alloc: BitAllocator,
}

impl GrowBitAllocator {
    pub const fn empty() -> Self {
        Self {
            alloc: BitAllocator::empty(),
        }
    }

    pub fn new(cap: usize) -> Self {
        Self {
            alloc: BitAllocator::new(cap),
        }
    }

    pub fn allocate(&mut self) -> usize {
        if self.alloc.num_free() == 0 {
            self.alloc.grow();
            self.alloc.allocate().unwrap()
        } else {
            self.alloc.allocate().unwrap()
        }
    }

    pub fn free(&mut self, bit: usize) {
        self.alloc.free(bit)
    }
}
