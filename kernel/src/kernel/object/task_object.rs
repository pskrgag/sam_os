use crate::kernel::locking::mutex::*;
use crate::kernel::locking::spinlock::*;
use crate::kernel::object::capabilities::CapabilityMask;
use crate::kernel::object::factory_object::FACTORY;
use crate::kernel::object::handle::Handle;
use crate::kernel::object::handle_table::HandleTable;
use crate::kernel::object::thread_object::Thread;
use crate::kernel::object::vms_object::Vms;
use crate::kernel::object::KernelObjectBase;
use crate::kernel::tasks::task::TaskInner;
use rtl::error::ErrorType;
use rtl::handle::HandleBase;
use rtl::handle::HANDLE_INVALID;
use hal::address::VirtAddr;

use alloc::string::String;
use alloc::sync::Arc;

use object_lib::object;

#[derive(object)]
pub struct Task {
    inner: Spinlock<TaskInner>,
    name: String,
    id: u32,
    vms: Arc<Vms>,
    handles: Mutex<HandleTable>,
    base: KernelObjectBase,
}

impl Task {
    pub fn new_kernel() -> Option<Arc<Task>> {
        Some(
            Arc::try_new(Self {
                inner: Spinlock::new(TaskInner::new_user()),
                name: "kernel task".into(),
                id: 0,
                vms: Vms::new_kernel()?,
                handles: Mutex::new(HandleTable::new()),
                base: KernelObjectBase::new(),
            })
            .ok()?,
        )
    }

    pub fn new(name: String) -> Option<Arc<Task>> {
        Some(
            Arc::try_new(Self {
                inner: Spinlock::new(TaskInner::new_user()),
                name,
                id: 0,
                vms: Vms::new_user()?,
                handles: Mutex::new(HandleTable::new()),
                base: KernelObjectBase::new(),
            })
            .ok()?,
        )
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

    pub fn add_initial_thread(self: &Arc<Self>, t: Arc<Thread>, boot_handle: HandleBase) {
        let mut table = self.handle_table();

        t.setup_args(&[
            table.add(Handle::new(self.vms().clone(), Vms::full_caps())),
            table.add(Handle::new(FACTORY.clone(), CapabilityMask::any())),
            boot_handle,
        ]);
        self.inner.lock().add_thread(t);
    }

    pub fn start_inner(&self) {
        self.inner.lock().start();
    }

    pub fn start(self: Arc<Self>, ep: VirtAddr, obj: Option<Handle>) -> Result<(), ErrorType> {
        use core::sync::atomic::{AtomicU16, Ordering};

        static ID_THREAD: AtomicU16 = AtomicU16::new(1);

        let init_thread = Thread::new(self.clone(), ID_THREAD.fetch_add(1, Ordering::Relaxed))
            .ok_or(ErrorType::NoMemory)?;
        let mut boot_handle: HandleBase = HANDLE_INVALID;

        if let Some(obj) = obj {
            let mut new_table = self.handle_table();
            boot_handle = new_table.add(obj);
        }

        init_thread.init_user(ep);
        self.add_initial_thread(init_thread, boot_handle);

        self.start_inner();
        Ok(())
    }

    pub fn vms_handle(&self) -> HandleBase {
        self.handle_table()
            .add(Handle::new(self.vms().clone(), Vms::full_caps()))
    }
}
