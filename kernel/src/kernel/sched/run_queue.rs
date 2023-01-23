use crate::kernel::{locking::fake_lock::FakeLock, threading::thread::ThreadRef};
use crate::percpu_global;
use alloc::collections::linked_list::LinkedList;

pub struct RunQueue {
    list: LinkedList<ThreadRef>,
    cur: Option<u16>,
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
        self.cur = Some(next.read().id());

        Some(next)
    }

    pub fn empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn current_id(&self) -> Option<u16> {
        self.cur
    }
}
