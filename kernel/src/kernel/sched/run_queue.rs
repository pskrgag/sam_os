use crate::kernel::{locking::fake_lock::FakeLock, tasks::thread::{ThreadRef, ThreadWeakRef}};
use crate::percpu_global;
use alloc::collections::linked_list::LinkedList;

use alloc::sync::Arc;

pub struct RunQueue {
    list: LinkedList<ThreadRef>,
    cur: Option<ThreadWeakRef>,
}

percpu_global!(
    pub static RUN_QUEUE: FakeLock<RunQueue> = FakeLock::new(RunQueue::new());
);

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            list: LinkedList::new(),
            cur: None,
        }
    }

    pub fn add(&mut self, t: ThreadRef) {
        self.list.push_back(t);
    }

    pub fn pop(&mut self) -> Option<ThreadRef> {
        let next = self.list.pop_front()?;
        self.cur = Some(Arc::downgrade(&next));
        Some(next)
    }

    pub fn empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn current(&self) -> Option<ThreadRef> {
        let a = self.cur.as_ref()?;

        a.upgrade()
    }
}
