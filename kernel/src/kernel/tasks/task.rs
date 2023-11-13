use object_lib::object;
use crate::{
    kernel::{
        object::handle_table::HandleTable,
        tasks::thread::Thread,
    },
    mm::vms::{Vms, VmsRef},
};
use alloc::{
    collections::LinkedList,
    string::String,
    sync::Weak,
};
use spin::Once;

#[derive(object)]
pub struct TaskObject {
    name: String,
    id: u32,
    handles: HandleTable,
    threads: LinkedList<Weak<Thread>>,
    vms: VmsRef,
}

/* ToDo: kernel task is redundant and should be dropped at all,
 *
 * I have idle threads for sched testing purposes, so let me leave things
 * as-is for now....
 */
static KERNEL_TASK: Once<TaskObjectRef> = Once::new();
static INIT_TASK: Once<TaskObjectRef> = Once::new();

impl TaskObject {
    pub fn new_kernel(name: String) -> TaskObjectRef {
        Self::construct(Self {
            name: name,
            id: 0,
            handles: HandleTable::new(),
            threads: LinkedList::new(),
            vms: Vms::new_kernel(),
        })
    }

    pub fn new_user(name: String) -> TaskObjectRef {
        Self::construct(Self {
            name: name,
            id: 0,
            handles: HandleTable::new(),
            threads: LinkedList::new(),
            vms: Vms::new_user(),
        })
    }

    pub fn add_thread(&mut self, t: Weak<Thread>) {
        self.threads.push_back(t);
    }

    pub fn vms(&self) -> VmsRef {
        self.vms.clone()
    }
}

pub fn init_kernel_task() {
    KERNEL_TASK.call_once(|| TaskObject::new_kernel("kernel task".into()));
    INIT_TASK.call_once(|| TaskObject::new_user("init task".into()));
}

/// NOTE: init_kernel_task() should be called before this
/// Anyway, the only thing caller may do in case of failure is panic
///
/// .get_unchecked() is too error-prone, IMO
pub fn kernel_task() -> TaskObjectRef {
    KERNEL_TASK.get().unwrap().clone()
}

/// NOTE: init_kernel_task() should be called before this
/// Anyway, the only thing caller may do in case of failure is panic
///
/// .get_unchecked() is too error-prone, IMO
pub fn init_task() -> TaskObjectRef {
    INIT_TASK.get().unwrap().clone()
}
