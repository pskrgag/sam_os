use crate::kernel::{locking::fake_lock::FakeLock, tasks::thread::{Thread, ThreadRef}};
use crate::percpu_global;
use alloc::collections::linked_list::LinkedList;

use alloc::sync::{Arc, Weak};

pub struct RunQueue {
    list: LinkedList<Arc<Thread>>,
    cur: Option<Weak<Thread>>,
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

    pub fn add(&mut self, t: Arc<Thread>) {
        self.list.push_back(t);
    }

    pub fn pop(&mut self) -> Arc<Thread> {
        let next = self.list.pop_front().expect("Calling pop on empty queue is a bug");
        self.cur = Some(Arc::downgrade(&next));
        next
    }

    pub fn empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn current(&self) -> Option<Arc<Thread>> {
        let a = self.cur.as_ref()?;

        a.upgrade()
    }
}
