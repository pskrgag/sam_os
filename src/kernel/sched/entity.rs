use crate::kernel::threading::{thread::Thread, ThreadRef};
use alloc::sync::Arc;

use qrwlock::qrwlock::RwLock;

pub struct SchedEntity {
    thread: ThreadRef,
}

impl SchedEntity {
    pub fn new(thread: ThreadRef) -> Self {
        Self { thread: thread }
    }

    pub fn thread(&self) -> &RwLock<Thread> {
        self.thread.as_ref()
    }
}
