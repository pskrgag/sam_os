use crate::mm::vmm::vms::Vms;
use crate::object::capabilities::CapabilityMask;
use crate::object::factory_object::FACTORY;
use crate::object::handle::Handle;
use crate::object::handle_table::HandleTable;
use crate::object::KernelObjectBase;
use crate::sync::{mutex::MutexGuard, Mutex, Spinlock};
use crate::tasks::thread::Thread;
use alloc::collections::LinkedList;
use hal::address::VirtAddr;
use rtl::error::ErrorType;
use rtl::handle::HandleBase;
use rtl::handle::HANDLE_INVALID;
use rtl::signal::Signal;
use spin::Once;

use alloc::string::String;
use alloc::sync::Arc;

pub struct TaskInner {
    threads: LinkedList<Arc<Thread>>,
}

static INIT_TASK: Once<Arc<Task>> = Once::new();
static KERNEL_TASK: Once<Arc<Task>> = Once::new();

impl TaskInner {
    pub fn new_user() -> Self {
        Self {
            threads: LinkedList::new(),
        }
    }

    pub fn add_thread(&mut self, t: Arc<Thread>) {
        self.threads.push_back(t);
    }

    pub fn start(&mut self) -> Result<(), ErrorType> {
        self.threads.front().unwrap().start()
    }
}

pub fn init_task() -> Arc<Task> {
    INIT_TASK.call_once(|| Task::new("init".into()).expect("No memory for initial task"));
    INIT_TASK.get().unwrap().clone()
}

pub fn kernel_task() -> Arc<Task> {
    KERNEL_TASK.call_once(|| Task::new_kernel().expect("No memory for kernel task"));
    KERNEL_TASK.get().unwrap().clone()
}

pub struct Task {
    inner: Spinlock<TaskInner>,
    name: String,
    id: u32,
    vms: Arc<Vms>,
    handles: Mutex<HandleTable>,
    base: KernelObjectBase,
}

crate::kernel_object!(Task, Signal::None.into());

impl Task {
    pub fn new_kernel() -> Option<Arc<Task>> {
        Arc::try_new(Self {
            inner: Spinlock::new(TaskInner::new_user()),
            name: "kernel task".into(),
            id: 0,
            vms: Vms::new_kernel()?,
            handles: Mutex::new(HandleTable::new()),
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn new(name: String) -> Option<Arc<Task>> {
        Arc::try_new(Self {
            inner: Spinlock::new(TaskInner::new_user()),
            name,
            id: 0,
            vms: Vms::new_user()?,
            handles: Mutex::new(HandleTable::new()),
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn handle_table<'a>(&'a self) -> MutexGuard<'a, HandleTable> {
        self.handles.lock()
    }

    pub fn vms(&self) -> &Arc<Vms> {
        &self.vms
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_thread(&self, t: Arc<Thread>) {
        self.inner.lock().add_thread(t);
    }

    fn start_inner(&self) -> Result<(), ErrorType> {
        self.inner.lock().start()
    }

    pub fn start(self: Arc<Self>, ep: VirtAddr, obj: Option<Handle>) -> Result<(), ErrorType> {
        use core::sync::atomic::{AtomicU16, Ordering};

        static ID_THREAD: AtomicU16 = AtomicU16::new(1);

        let init_thread = Thread::new_user(self.clone(), ID_THREAD.fetch_add(1, Ordering::Relaxed))
            .ok_or(ErrorType::NoMemory)?;
        let mut boot_handle: HandleBase = HANDLE_INVALID;

        if let Some(obj) = obj {
            let mut new_table = self.handle_table();
            boot_handle = new_table.add(obj);
        }

        let mut table = self.handle_table();
        init_thread.init_user(
            ep,
            Some([
                table.add(Handle::new(self.vms().clone(), Vms::full_caps())),
                table.add(Handle::new(FACTORY.clone(), CapabilityMask::any())),
                boot_handle,
            ]),
        );
        self.inner.lock().add_thread(init_thread);
        self.start_inner()
    }

    pub fn vms_handle(&self) -> HandleBase {
        self.handle_table()
            .add(Handle::new(self.vms().clone(), Vms::full_caps()))
    }
}
