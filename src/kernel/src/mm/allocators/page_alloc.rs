use crate::{arch, kernel, kernel::locking::spinlock::*};
use alloc::vec::Vec;
use bitmaps::Bitmap;
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::*;

pub struct PageAlloc {
    pool: Vec<Bitmap<64>>,
    start: Pfn,
}

unsafe impl Send for PageAlloc {}

pub static PAGE_ALLOC: Spinlock<PageAlloc> = Spinlock::new(PageAlloc::default());

pub fn page_allocator() -> SpinlockGuard<'static, PageAlloc> {
    PAGE_ALLOC.lock()
}

impl PageAlloc {
    pub const fn default() -> Self {
        Self {
            start: Pfn::new(0x0),
            pool: Vec::new(),
        }
    }

    pub fn new(start: PhysAddr, mut size: usize) -> Option<Self> {
        let pool_size = *size.round_up_page() / PAGE_SIZE / 64;
        let mut pool = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            pool.push(Bitmap::<64>::new());
        }

        println!("Page allocator initialized: phys size {:x}", size);

        Some(Self {
            pool,
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

    pub fn alloc(&mut self, num: usize) -> Option<PhysAddr> {
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

                return Some(PhysAddr::from(
                    self.bitmap_to_pfn(bitmap.unwrap(), idx.unwrap()),
                ));
            }

            bitmap_idx += 1;
        }

        None
    }

    pub fn free(&mut self, start: PhysAddr, num: usize) {
        let pfn: Pfn = start.into();
        let (bitmap, idx) = ((pfn - self.start) / 64, (pfn - self.start) % 64);

        for i in 0..num {
            self.pool[(bitmap + (idx + i) % 64) as usize].set(((idx + i) % 64) as usize, false);
        }
    }
}

pub fn init() {
    let alloc_start = PhysAddr::from(kernel::misc::image_end_rounded());
    let alloc_size = arch::ram_size() as usize - kernel::misc::image_size();

    println!(
        "Page allocator start {:x} size {:x}",
        alloc_start.get(),
        alloc_size
    );

    *PAGE_ALLOC.lock() = PageAlloc::new(alloc_start, alloc_size as usize).unwrap();
}
