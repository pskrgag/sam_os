use crate::mm::memset_pages;
use crate::mm::pmm::page::Page;
use crate::mm::pmm::page_list::PageList;
use crate::mm::pmm::phys_layout::phys_info;
use crate::sync::{Spinlock, spinlock::SpinlockGuard};
use hal::address::*;

pub struct PageAlloc {
    list: PageList,
}

pub static PAGE_ALLOC: Spinlock<PageAlloc> = Spinlock::new(PageAlloc::default());

pub fn page_allocator() -> SpinlockGuard<'static, PageAlloc> {
    PAGE_ALLOC.lock()
}

impl PageAlloc {
    const fn default() -> Self {
        Self {
            list: PageList::default(),
        }
    }

    fn alloc_page(&mut self) -> Option<Page> {
        let next = self.list.pop_front_with_cb(|page| {
            assert!(page.is_free());
            page.mark_occupied();
        })?;

        unsafe {
            memset_pages(next.0, 1);
        }

        Some(next)
    }

    pub fn alloc_pages(&mut self, count: usize) -> Option<PageList> {
        let mut list = PageList::default();

        for _ in 0..count {
            match self.alloc_page() {
                Some(next) => {
                    list.push_back(next);
                }
                None => {
                    self.free(list);
                    return None;
                }
            }
        }

        Some(list)
    }

    pub fn alloc_contigious(&mut self, count: usize) -> Option<PhysAddr> {
        let mut found_pfn: Option<Pfn> = None;

        'outer: for reg in phys_info().regions() {
            if count > reg.size() / hal::arch::PAGE_SIZE {
                continue;
            }

            let mut candidate: Option<Pfn> = None;
            let mut found = 0;

            for pfn in reg.pfn_range() {
                unsafe {
                    let page = super::page_list::pfn_to_halpage(pfn);

                    if page.as_ref().is_free() {
                        if candidate.is_none() {
                            candidate = Some(pfn);
                            found = 1;
                        } else {
                            found += 1;
                        }
                    } else {
                        candidate = None;
                        found = 0;
                    }

                    if found == count {
                        found_pfn = candidate;
                        break 'outer;
                    }
                }
            }
        }

        if let Some(pfn) = found_pfn {
            for i in 0..count {
                unsafe {
                    self.list.remove_pfn_with_cb(pfn + i, |page| {
                        assert!(page.is_free());
                        page.mark_occupied();
                    });
                }
            }
        }

        found_pfn.map(|x| x.into())
    }

    pub fn free_contig(&mut self, pa: PhysAddr, count: usize) {
        let pfn: Pfn = pa.into();

        for i in 0..count {
            self.list.push_back_with_cb(Page(pfn + i), |page| {
                assert!(!page.is_free());
                page.mark_free();
            });
        }
    }

    pub fn free(&mut self, mut list: PageList) {
        while let Some(next) = list.pop_front() {
            self.list.push_front_with_cb(next, |page| {
                assert!(!page.is_free());
                page.mark_free();
            });
        }
    }
}

pub fn init() {
    let mut allocator = PAGE_ALLOC.lock();
    let mut list = PageList::default();

    for reg in phys_info().regions() {
        info!(
            "Page allocator region {:x} size {:x}\n",
            reg.start, reg.size
        );

        for i in reg.pfn_range() {
            list.push_back_with_cb(Page(i), |page| {
                page.mark_free();
            });
        }
    }

    allocator.list = list;
}
