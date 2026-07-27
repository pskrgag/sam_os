use super::list::ListNode;
use core::sync::atomic::AtomicUsize;

#[repr(u8)]
#[derive(PartialEq)]
pub enum PageState {
    Free,
    Occupied,
}

#[repr(C)]
pub struct Page {
    pub refcount: AtomicUsize,
    pub list: ListNode,
    pub state: PageState,
}

impl Page {
    /// # Safety
    ///
    /// [`node`] must be part of the list.
    pub unsafe fn from_node(node: &'static mut ListNode) -> &'static mut Self {
        let mut node = node as *const _ as usize;

        node -= core::mem::offset_of!(Page, list);

        unsafe { &mut *(node as *mut u8 as *mut Self) }
    }

    pub fn next_page(&'static mut self) -> Option<&'static mut Self> {
        unsafe { self.list.next_node().map(|x| Self::from_node(x)) }
    }

    pub fn prev_page(&'static mut self) -> Option<&'static mut Self> {
        unsafe { self.list.prev_node().map(|x| Self::from_node(x)) }
    }

    pub fn remove(&'static mut self) -> Option<&'static mut Self> {
        unsafe { self.list.remove().map(|x| Self::from_node(x)) }
    }

    pub fn is_free(&self) -> bool {
        self.state == PageState::Free
    }

    pub fn mark_occupied(&mut self) {
        self.state = PageState::Occupied;
    }

    pub fn mark_free(&mut self) {
        self.state = PageState::Free;
    }
}
