//! Bitmap allocator

use alloc::vec::Vec;
use rtl::error::ErrorType;

pub struct BitAllocator {
    bits: Vec<u8>,
    bit_capacity: usize,
    bits_left: usize,
    // TODO: add hint for allocation
}

impl BitAllocator {
    pub fn new(count: usize) -> Self {
        Self {
            bits: alloc::vec![0; count / 8],
            bit_capacity: count,
            bits_left: count,
        }
    }

    pub fn allocate_specific(&mut self, bit: usize) -> Result<usize, ErrorType> {
        let idx = bit / 8;
        let offset = bit % 8;

        if bit < self.bit_capacity && self.bits[idx] & (1 << offset) == 0 {
            self.bits[idx] |= 1 << offset;
            Ok(bit)
        } else {
            Err(ErrorType::InvalidArgument)
        }
    }

    pub fn allocate(&mut self) -> Option<usize> {
        if self.bits_left != 0 {
            for (i, entry) in self.bits.iter_mut().enumerate() {
                if *entry != u8::MAX {
                    let ones = entry.trailing_ones() as usize;

                    *entry |= 1 << ones;
                    self.bits_left -= 1;
                    return Some(i * 8 + ones);
                }
            }

            panic!("Unreachable")
        } else {
            None
        }
    }

    pub fn free(&mut self, bit: usize) {
        let idx = bit / 8;
        let offset = bit % 8;

        assert!(bit < self.bit_capacity);
        assert_ne!(self.bits[idx] & (1 << offset), 0);

        self.bits[idx] &= !(1 << offset);
    }
}
