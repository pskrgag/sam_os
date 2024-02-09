use crate::kernel::object::thread_object::Thread;
use alloc::collections::linked_list::LinkedList;

use alloc::sync::Arc;

pub struct RunQueue {
    list_active: LinkedList<Arc<Thread>>,
    list_sleep: LinkedList<Arc<Thread>>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            list_active: LinkedList::new(),
            list_sleep: LinkedList::new(),
        }
    }

    pub fn add_running(&mut self, t: Arc<Thread>) {
        self.list_active.push_back(t);
    }

    pub fn add_sleeping(&mut self, t: Arc<Thread>) {
        self.list_sleep.push_back(t);
    }

    pub fn move_by_pred<F: Fn(&mut Arc<Thread>) -> bool>(&mut self, f: F) {
        let mut ext = self.list_sleep.extract_if(f).collect::<LinkedList<_>>();

        self.list_active.append(&mut ext);
    }

    pub fn pop(&mut self) -> Option<Arc<Thread>> {
        let next = self.list_active.pop_front();
        next
    }

    pub fn empty(&self) -> bool {
        self.list_active.is_empty()
    }
}
