//! Page list

use super::page::page_array_base;
use super::page::Page;
use super::phys_layout::phys_info;
use core::marker::PhantomData;
use core::ptr::NonNull;
use hal::address::Pfn;
use hal::page::Page as HalPage;

pub struct PageList {
    start: Option<NonNull<HalPage>>,
    tail: Option<NonNull<HalPage>>,
    count: usize,
}

unsafe impl Send for PageList {}

pub struct PageListIterator<'a> {
    current: Option<NonNull<HalPage>>,
    count: usize,
    _pd: PhantomData<&'a PageList>,
}

impl Iterator for PageListIterator<'_> {
    type Item = Pfn;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut res = self.current.take()?;
            let next_page = res.as_mut().list.next_node();

            self.current = next_page.map(|x| NonNull::new_unchecked(HalPage::from_node(x)));
            self.count -= 1;
            Some(halpage_to_pfn(res))
        }
    }
}

impl PageListIterator<'_> {
    pub fn pages(&self) -> usize {
        self.count
    }
}

impl PageList {
    pub const fn default() -> Self {
        Self {
            start: None,
            tail: None,
            count: 0,
        }
    }

    pub fn push_back_with_cb<F: FnMut(&mut HalPage)>(&mut self, page: Page, mut cb: F) {
        unsafe {
            let mut page = page_to_halpage(&page);

            cb(page.as_mut());

            if let Some(mut tail) = self.tail.take() {
                tail.as_mut().list.push(&mut page.as_mut().list);
                self.tail = Some(page);
            } else {
                self.tail = Some(page);
            }

            if self.start.is_none() {
                self.start = self.tail;
            }

            self.count += 1;
        }
    }

    pub unsafe fn remove_pfn_with_cb<F: FnMut(&mut HalPage)>(&mut self, pfn: Pfn, mut cb: F) {
        unsafe {
            assert!(self.count != 0);

            let mut page = page_to_halpage(&Page(pfn));
            let prev = page.as_mut().prev_page();
            let next = page.as_mut().remove();

            cb(page.as_mut());

            if next.is_none() {
                assert!(self.tail == Some(page));

                self.tail = prev.map(|x| NonNull::new_unchecked(x));
            }

            if Some(page) == self.start {
                self.start = next.map(|x| NonNull::new_unchecked(x))
            }

            self.count -= 1;
        }
    }

    pub fn push_back(&mut self, page: Page) {
        self.push_back_with_cb(page, |_| {})
    }

    pub fn pop_front_with_cb<F: FnMut(&mut HalPage)>(&mut self, mut cb: F) -> Option<Page> {
        unsafe {
            let mut start = self.start.take()?;

            cb(start.as_mut());
            assert!(self.count >= 1);

            self.start = start
                .as_mut()
                .list
                .remove()
                .map(|x| NonNull::new_unchecked(HalPage::from_node(x)));

            self.count -= 1;

            if self.count == 0 {
                self.tail = None;
            }

            Some(Page(halpage_to_pfn(start)))
        }
    }

    pub fn pop_front(&mut self) -> Option<Page> {
        self.pop_front_with_cb(|_| {})
    }

    pub fn push_front_with_cb<F: FnMut(&mut HalPage)>(&mut self, page: Page, mut cb: F) {
        unsafe {
            let mut page = page_to_halpage(&page);

            cb(page.as_mut());

            if let Some(mut start) = self.start.take() {
                start.as_mut().list.prepend(&mut page.as_mut().list);
                self.start = Some(page);
            } else {
                self.start = Some(page);
            }

            if self.tail.is_none() {
                self.tail = self.start;
            }

            self.count += 1;
        }
    }

    pub fn push_front(&mut self, page: Page) {
        self.push_front_with_cb(page, |_| {});
    }

    pub fn iter(&self) -> PageListIterator<'_> {
        PageListIterator {
            current: self.start,
            count: self.count,
            _pd: PhantomData,
        }
    }

    pub fn pages(&self) -> usize {
        self.count
    }
}

fn halpage_to_pfn(owned: NonNull<HalPage>) -> Pfn {
    unsafe {
        let info = phys_info();
        let diff = owned.offset_from(page_array_base());

        assert!(diff >= 0);

        info.lowest_pfn() + diff as usize
    }
}

fn page_to_halpage(owned: &Page) -> NonNull<HalPage> {
    let info = phys_info();
    let diff = owned.0 - info.lowest_pfn();

    assert!(info.lowest_pfn() <= owned.0 && owned.0 < info.highest_pfn());

    unsafe { page_array_base().add(diff) }
}

pub(crate) unsafe fn pfn_to_halpage(pfn: Pfn) -> NonNull<HalPage> {
    page_to_halpage(&Page(pfn))
}
