//! Intrusive linked list

use core::ptr::NonNull;

#[derive(Default)]
pub struct ListNode {
    next: Option<NonNull<ListNode>>,
    prev: Option<NonNull<ListNode>>,
}

impl ListNode {
    pub fn next_node(&'static mut self) -> Option<&'static mut ListNode> {
        unsafe { self.next.map(|mut x| x.as_mut()) }
    }

    pub fn prev_node(&'static mut self) -> Option<&'static mut ListNode> {
        unsafe { self.prev.map(|mut x| x.as_mut()) }
    }

    fn is_unlinked(&self) -> bool {
        self.next.is_none() && self.prev.is_none()
    }

    /// Initializes node to default value
    pub fn init(&'static mut self) {
        *self = ListNode::default();
    }

    /// Inserts node before current one
    pub fn prepend(&'static mut self, other: &'static mut ListNode) {
        assert!(other.is_unlinked());

        let prev = self.prev;

        unsafe {
            // 1. list current node
            other.prev = prev;
            other.next = Some(NonNull::new_unchecked(self as _));

            // 2. relink next
            if let Some(mut prev) = prev {
                prev.as_mut().next = Some(NonNull::new_unchecked(other as _))
            }

            // 3. relink current
            self.prev = Some(NonNull::new_unchecked(other as _))
        }
    }

    /// Inserts node after current one
    pub fn push(&'static mut self, other: &'static mut ListNode) {
        assert!(other.is_unlinked());

        let next = self.next;

        unsafe {
            // 1. list current node
            other.next = next;
            other.prev = Some(NonNull::new_unchecked(self as _));

            // 2. relink next
            if let Some(mut next) = next {
                next.as_mut().prev = Some(NonNull::new_unchecked(other as _))
            }

            // 3. relink current
            self.next = Some(NonNull::new_unchecked(other as _))
        }
    }

    /// Removes a node and returns next one
    pub fn remove(&'static mut self) -> Option<&'static mut ListNode> {
        let prev = self.prev;
        let next = self.next;

        unsafe {
            if let Some(mut prev) = prev {
                prev.as_mut().next = next;
            }

            if let Some(mut next) = next {
                next.as_mut().prev = prev;
            }

            self.init();
            next.map(|mut x| x.as_mut())
        }
    }
}
