use crate::kernel::object::thread_object::Thread;
use crate::percpu_global;
use alloc::collections::linked_list::LinkedList;

use alloc::sync::Arc;
use rtl::locking::fake_lock::FakeLock;

pub struct RunQueue {
    list: LinkedList<Arc<Thread>>,
}

percpu_global!(
    pub static RUN_QUEUE: FakeLock<RunQueue> = FakeLock::new(RunQueue::new());
);

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    pub fn add(&mut self, t: Arc<Thread>) {
        self.list.push_back(t);
    }

    pub fn pop(&mut self) -> Arc<Thread> {
        let next = self
            .list
            .pop_front()
            .expect("Calling pop on empty queue is a bug");
        next
    }

    pub fn empty(&self) -> bool {
        self.list.is_empty()
    }
}
