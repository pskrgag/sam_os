use crate::kernel::object::handle::Handle;
use crate::{
    kernel::{
        locking::spinlock::{Spinlock, SpinlockGuard},
        object::handle_table::HandleTable,
        tasks::thread::Thread,
    },
    mm::vms::Vms,
};
use alloc::{collections::LinkedList, string::String};
use object_lib::object;
use spin::Once;

pub struct TaskObject {
    threads: LinkedList<Arc<Thread>>,
}

#[derive(object)]
pub struct Task {
    inner: Spinlock<TaskObject>,
    name: String,
    id: u32,
    vms: Arc<Vms>,
    handles: Spinlock<HandleTable>,
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
        let s = Arc::new(Self {
            inner: Spinlock::new(TaskObject::new_user()),
            name,
            id: 0,
            vms: Vms::new_user(),
            handles: Spinlock::new(HandleTable::new()),
        });

        let handle = Handle::new::<Task>(s.clone());
        s.handle_table().add(handle);

        // let handle = Handle::new::<Vms>(s.vms.clone());
        // s.add_handle(handle);

        s
    }

    pub fn handle_table(&self) -> SpinlockGuard<HandleTable> {
        self.handles.lock()
    }

    pub fn vms(&self) -> Arc<Vms> {
        self.vms.clone()
    }

    pub fn add_thread(&self, t: Arc<Thread>) {
        self.inner.lock().add_thread(t);
    }

    pub fn start(&self) {
        self.inner.lock().start()
    }
}

impl TaskObject {
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
