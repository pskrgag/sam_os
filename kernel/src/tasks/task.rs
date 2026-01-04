use super::handle_page::{HandleName, HandlePage};
use crate::mm::vmm::vms::Vms;
use crate::object::KernelObjectBase;
use crate::object::capabilities::CapabilityMask;
use crate::object::factory_object::FACTORY;
use crate::object::handle::Handle;
use crate::object::handle_table::HandleTable;
use crate::sched::{current, current_task};
use crate::sync::{Mutex, Spinlock, async_mutex::MutexGuard};
use crate::tasks::thread::Thread;
use alloc::collections::LinkedList;
use alloc::sync::Arc;
use hal::address::VirtAddr;
use heapless::String;
use rtl::error::ErrorType;
use rtl::handle::HandleBase;
use rtl::signal::Signal;
use spin::Once;

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
    INIT_TASK.call_once(|| {
        Task::new(TaskName::try_from("init").unwrap()).expect("No memory for initial task")
    });
    INIT_TASK.get().unwrap().clone()
}

pub fn kernel_task() -> Arc<Task> {
    KERNEL_TASK.call_once(|| Task::new_kernel().expect("No memory for kernel task"));
    KERNEL_TASK.get().unwrap().clone()
}

pub type TaskName = String<100>;

pub struct Task {
    inner: Spinlock<TaskInner>,
    name: TaskName,
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
            name: TaskName::try_from("kernel task").unwrap(),
            id: 0,
            vms: Vms::new_kernel()?,
            handles: Mutex::new(HandleTable::new()),
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn new(name: TaskName) -> Option<Arc<Task>> {
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

    pub async fn handle_table<'a>(&'a self) -> Result<MutexGuard<'a, HandleTable>, ErrorType> {
        self.handles.lock().await
    }

    pub fn vms(&self) -> &Arc<Vms> {
        &self.vms
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn add_thread(&self, t: Arc<Thread>) {
        self.inner.lock().add_thread(t);
    }

    fn start_inner(&self) -> Result<(), ErrorType> {
        self.inner.lock().start()
    }

    pub async fn start(
        self: Arc<Self>,
        ep: VirtAddr,
        obj: Option<Handle>,
    ) -> Result<(), ErrorType> {
        use core::sync::atomic::{AtomicU16, Ordering};

        static ID_THREAD: AtomicU16 = AtomicU16::new(1);

        let init_thread = Thread::new_user(self.clone(), ID_THREAD.fetch_add(1, Ordering::Relaxed))
            .await
            .ok_or(ErrorType::NoMemory)?;
        let mut handle_page = HandlePage::new(self.clone());

        handle_page
            .push(
                Handle::new(self.vms().clone(), Vms::full_caps()),
                HandleName::try_from("VMS").unwrap(),
            )
            .await?;

        handle_page
            .push(
                Handle::new(FACTORY.clone(), CapabilityMask::any()),
                HandleName::try_from("FACTORY").unwrap(),
            )
            .await?;

        if let Some(obj) = obj {
            handle_page
                .push(obj, HandleName::try_from("BOOT").unwrap())
                .await?;
        }

        init_thread
            .init_user(ep, Some(handle_page.into_ptr().await? as usize))
            .await;
        self.inner.lock().add_thread(init_thread);
        self.start_inner()
    }

    pub fn with_attached_task<F: FnOnce()>(self: Arc<Self>, f: F) {
        current().with_disabled_preemption(|| {
            let cur_task = current_task();

            // TODO: disallow nested switching
            self.vms().switch_to();
            f();
            cur_task.vms().switch_to();
        });
    }

    pub async fn vms_handle(&self) -> Result<HandleBase, ErrorType> {
        Ok(self
            .handle_table()
            .await?
            .add(Handle::new(self.vms().clone(), Vms::full_caps())))
    }
}
