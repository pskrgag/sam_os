use super::mutex::Mutex;
use crate::kernel::object::thread_object::Thread;
use crate::kernel::sched::current;
use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

pub struct WaitQueue<T> {
    data: Mutex<VecDeque<T>>,
    waiters: Mutex<Vec<Weak<Thread>>>,
}

impl<T> WaitQueue<T> {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(VecDeque::new()),
            waiters: Mutex::new(Vec::new()),
        }
    }

    pub fn produce(&self, data: T) {
        self.data.lock().push_back(data);

        for waiter in &*self.waiters.lock() {
            if let Some(waiter) = waiter.upgrade() {
                waiter.wake();
            }
        }
    }

    pub fn consume(&self) -> T {
        let cur = current().unwrap();

        loop {
            let mut data = self.data.lock();

            if let Some(res) = data.pop_front() {
                break res;
            } else {
                self.waiters.lock().push(Arc::downgrade(&cur));

                drop(data);
                // cur.sleep(ThreadSleepReason::WaitQueue);
            }
        }
    }
}
