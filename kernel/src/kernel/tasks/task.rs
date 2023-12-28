use crate::kernel::object::task_object::Task;
use crate::kernel::object::thread_object::Thread;
use alloc::collections::LinkedList;
use spin::Once;

use alloc::sync::Arc;

pub struct TaskInner {
    threads: LinkedList<Arc<Thread>>,
}

/* ToDo: kernel task is redundant and should be dropped at all,
 *
 * I have idle threads for sched testing purposes, so let me leave things
 * as-is for now....
 */
static KERNEL_TASK: Once<Arc<Task>> = Once::new();
static INIT_TASK: Once<Arc<Task>> = Once::new();

impl TaskInner {
    pub fn new_kernel() -> Self {
        Self {
            threads: LinkedList::new(),
        }
    }

    pub fn new_user() -> Self {
        Self {
            threads: LinkedList::new(),
        }
    }

    pub fn add_thread(&mut self, t: Arc<Thread>) {
        self.threads.push_back(t);
    }

    pub fn start(&mut self) {
        let t = self.threads.front().unwrap();
        t.start();
    }
}

pub fn init_kernel_task() {
    KERNEL_TASK.call_once(|| Task::new("kernel task".into()));
}

/// NOTE: init_kernel_task() should be called before this
/// Anyway, the only thing caller may do in case of failure is panic
///
/// .get_unchecked() is too error-prone, IMO
pub fn kernel_task() -> Arc<Task> {
    KERNEL_TASK.get().unwrap().clone()
}

/// NOTE: init_kernel_task() should be called before this
/// Anyway, the only thing caller may do in case of failure is panic
///
/// .get_unchecked() is too error-prone, IMO
pub fn init_task() -> Arc<Task> {
    INIT_TASK.call_once(|| Task::new("init task".into()));
    INIT_TASK.get().unwrap().clone()
}
