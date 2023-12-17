use crate::{
    kernel::{
        locking::spinlock::Spinlock, object::handle_table::HandleTable, tasks::thread::Thread,
    },
    mm::vms::{Vms, VmsRef},
};
use alloc::{collections::LinkedList, string::String, sync::Weak};
use object_lib::object;
use spin::Once;
use crate::kernel::object::handle::Handle;

pub struct TaskObject {
    handles: HandleTable,
    threads: LinkedList<Weak<Thread>>,
}

#[derive(object)]
pub struct Task {
    inner: Spinlock<TaskObject>,
    name: String,
    id: u32,
    vms: VmsRef,
}

/* ToDo: kernel task is redundant and should be dropped at all,
 *
 * I have idle threads for sched testing purposes, so let me leave things
 * as-is for now....
 */
static KERNEL_TASK: Once<Arc<Task>> = Once::new();
static INIT_TASK: Once<Arc<Task>> = Once::new();

impl Task {
    pub fn new(name: String) -> Arc<Task> {
        let mut s = Arc::new(Self {
            inner: Spinlock::new(TaskObject::new_user()),
            name,
            id: 0,
            vms: Vms::new_user(),
        });

        let handle = Handle::new::<Task>(s.clone());
        s.add_handle(handle);

        s
    }

    pub fn add_handle(&self, h: Handle) {
        let mut i = self.inner.lock();
        i.add_handle(h);
    }

    pub fn vms(&self) -> VmsRef {
        self.vms.clone()
    }

    pub fn add_thread(&self, t: Weak<Thread>) {
        self.inner.lock().add_thread(t);
    }
}

impl TaskObject {
    pub fn new_kernel() -> Self {
        Self {
            handles: HandleTable::new(),
            threads: LinkedList::new(),
        }
    }

    pub fn new_user() -> Self {
        Self {
            handles: HandleTable::new(),
            threads: LinkedList::new(),
        }
    }

    pub fn add_handle(&mut self, h: Handle) {
        self.handles.add(h);
    }

    pub fn add_thread(&mut self, t: Weak<Thread>) {
        self.threads.push_back(t);
    }
}

pub fn init_kernel_task() {
    KERNEL_TASK.call_once(|| Task::new("kernel task".into()));
    INIT_TASK.call_once(|| Task::new("init task".into()));
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
    INIT_TASK.get().unwrap().clone()
}
