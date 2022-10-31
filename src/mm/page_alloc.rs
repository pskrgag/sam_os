// FIXME one day...
#[path = "../arch/aarch64/qemu/config.rs"]
mod config;

use bitmaps::Bitmap;
use core::mem::size_of;
use crate::{
    arch,
    kernel,
    mm::types::*,
    kernel::locking::spinlock::Spinlock,
    lib::collections::vector::Vector
};

pub struct PageAlloc
{
    pool: Vector<Bitmap<64>>,
    start: Pfn,
}

unsafe impl Send for PageAlloc { }

pub static PAGE_ALLOC: Spinlock<PageAlloc> = Spinlock::new(PageAlloc::default());

impl PageAlloc
{
    pub const fn default() -> Self {
        Self {
            start: Pfn::new(0x0),
            pool: Vector::new(),
        }
    }

    pub fn new(start: PhysAddr, size: usize) -> Option<Self> {
        let pool = Vector::with_capaicty(size / 64 + 1);

        if pool.is_none() {
            return None;
        }

        println!("Page allocator initialized: phys size {}", size);

        Some(Self {
            pool: pool.unwrap(),
            start: Pfn::from(start),
        })
    }

    fn mark_allocated(&mut self, bitmap1: usize, bitmap2: usize, idx1: usize, idx2: usize) {
        for i in bitmap1 + 1..bitmap2 {
            for j in 0..64 {
                self.pool[i].set(j, true);
            }
        }

        for i in idx1..64 {
            self.pool[bitmap1].set(i, true);
        }
        
        for i in idx2..64 {
            self.pool[bitmap2].set(i, true);
        }
    }

    #[inline]
    fn bitmap_to_pfn(&self, bitmap: usize, idx: usize) -> Pfn {
        Pfn::from(usize::from(self.start) + bitmap * 64 + idx)
    }

    pub fn alloc_pages(&mut self, num: usize) -> Option<PhysAddr> {
        let mut bitmap_idx: usize = 0;
        let mut cont_pages = 0;
        let (mut bitmap, mut idx) = (None, None);

        for i in &self.pool {
            if i.is_full() {
                cont_pages = 0;
                (bitmap, idx) = (None, None);
                continue;
            }

            /* We know it exists */
            let start = i.first_false_index().unwrap();
            if start != 0 {
                cont_pages = 0;
                (bitmap, idx) = (Some(bitmap_idx), Some(start));
            }

            let next = i.next_index(start);
            match next {
                Some(next) => {
                    cont_pages += next - start;
                },
                None => {
                    cont_pages += 64 - start;
                },
            }

            if cont_pages >= num {
                let next = if next.is_some() { next.unwrap() } else { 63 };
                self.mark_allocated(bitmap.unwrap(), bitmap_idx, idx.unwrap(), next);

                return Some(PhysAddr::from(self.bitmap_to_pfn(bitmap.unwrap(), idx.unwrap())));
            }

            bitmap_idx += 1;
        }

        None
    }

    pub fn free_pages(&mut self, start: PhysAddr, num: usize) {
        let pfn: Pfn = start.into();
        let (bitmap, idx) = ((pfn - self.start) / 64, (pfn - self.start) % 64);

        for i in 0..num{
            self.pool[(bitmap + (idx + i as u64) % 64) as usize].set(((idx + i as u64) % 64) as usize, false);
        }
    }
}

pub fn init() {
    let alloc_start = PhysAddr::from(usize::from(arch::ram_base()) + kernel::misc::image_size());
    let alloc_size = arch::ram_size() - kernel::misc::image_size();
    *PAGE_ALLOC.lock() = PageAlloc::new(alloc_start, alloc_size).unwrap();
}
