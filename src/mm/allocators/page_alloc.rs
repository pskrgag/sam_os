// FIXME one day...
#[path = "../../arch/aarch64/qemu/config.rs"]
mod config;

use crate::{
    arch, kernel, kernel::locking::spinlock::Spinlock, lib::collections::vector::Vector,
    mm::types::*,
};
use bitmaps::Bitmap;

pub struct PageAlloc {
    pool: Vector<Bitmap<64>>,
    start: Pfn,
}

unsafe impl Send for PageAlloc {}

pub static PAGE_ALLOC: Spinlock<PageAlloc> = Spinlock::new(PageAlloc::default());

impl PageAlloc {
    pub const fn default() -> Self {
        Self {
            start: Pfn::new(0x0),
            pool: Vector::new(),
        }
    }

    pub fn new(start: PhysAddr, size: usize) -> Option<Self> {
        let pool_size = size / 64 + 1;
        let pool = Vector::with_capaicty(pool_size);

        if pool.is_none() {
            return None;
        }

        let mut pool = pool.unwrap();

        for _ in 0..pool_size {
            pool.push(Bitmap::<64>::new());
        }

        println!("Page allocator initialized: phys size {}", size);

        Some(Self {
            pool: pool,
            start: Pfn::from(start),
        })
    }

    fn mark_allocated(&mut self, mut bitmap: usize, mut idx: usize, mut size: usize) {
        while {
            self.pool[bitmap].set(idx, true);

            idx = if idx == 63 {
                bitmap += 1;
                0
            } else {
                idx + 1
            };
            size -= 1;

            size != 0
        } {}
    }

    #[inline]
    fn bitmap_to_pfn(&self, bitmap: usize, idx: usize) -> Pfn {
        Pfn::from(usize::from(self.start) + bitmap * 64 + idx)
    }

    pub fn alloc_pages(&mut self, num: usize) -> Option<PhysAddr> {
        let mut bitmap_idx: usize = 0;
        let mut cont_pages = 0;
        let (mut bitmap, mut idx) = (Some(bitmap_idx), Some(0));

        for i in &self.pool {
            if i.is_full() {
                cont_pages = 0;
                (bitmap, idx) = (None, None);
                bitmap_idx += 1;
                continue;
            }

            /* We know it exists */
            let start = i.first_false_index().unwrap();
            if start != 0 {
                cont_pages = 0;
                (bitmap, idx) = (Some(bitmap_idx), Some(start));
            } else if idx.is_none() {
                cont_pages = 0;
                (bitmap, idx) = (Some(bitmap_idx), Some(start));
            }
            //println!("Dump phys  start {}", start);

            let next = i.next_index(start);
            match next {
                Some(next) => {
                    cont_pages += next - start;
                }
                None => {
                    cont_pages += 64 - start;
                }
            }

            if cont_pages >= num {
                self.mark_allocated(bitmap.unwrap(), idx.unwrap(), num);

                //println!("Dump phys 0x{:x}", PhysAddr::from(self.bitmap_to_pfn(bitmap.unwrap(), idx.unwrap())).get());

                return Some(PhysAddr::from(
                    self.bitmap_to_pfn(bitmap.unwrap(), idx.unwrap()),
                ));
            }

            bitmap_idx += 1;
        }

        None
    }

    pub fn free_pages(&mut self, start: PhysAddr, num: usize) {
        let pfn: Pfn = start.into();
        let (bitmap, idx) = ((pfn - self.start) / 64, (pfn - self.start) % 64);

        for i in 0..num {
            self.pool[(bitmap + (idx + i) % 64) as usize].set(((idx + i) % 64) as usize, false);
        }
    }
}

pub fn init() {
    let alloc_start = PhysAddr::from(arch::ram_base() as usize + kernel::misc::image_size());
    println!(
        "{} {}",
        arch::ram_size() as usize,
        kernel::misc::image_size()
    );
    let alloc_size = arch::ram_size() as usize - kernel::misc::image_size();

    println!(
        "Page allocator start {:x} size {:x}",
        alloc_start.get(),
        alloc_size
    );
    *PAGE_ALLOC.lock() = PageAlloc::new(alloc_start, alloc_size as usize).unwrap();
}
