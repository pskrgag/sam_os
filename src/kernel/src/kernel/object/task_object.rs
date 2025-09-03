use crate::kernel::locking::spinlock::*;
use crate::kernel::object::factory_object::Factory;
use crate::kernel::object::handle::Handle;
use crate::kernel::object::handle_table::HandleTable;
use crate::kernel::object::thread_object::Thread;
use crate::kernel::object::vms_object::Vms;
use crate::kernel::object::KernelObject;
use crate::kernel::tasks::task::TaskInner;
use rtl::error::ErrorType;
use rtl::handle::HandleBase;
use rtl::handle::HANDLE_INVALID;
use rtl::vmm::types::VirtAddr;

use alloc::string::String;
use alloc::sync::Arc;

use object_lib::object;

#[derive(object)]
pub struct Task {
    inner: Spinlock<TaskInner>,
    name: String,
    id: u32,
    vms: Arc<Vms>,
    handles: Spinlock<HandleTable>,
    factory: Arc<Factory>,

    // Cache handles to pass them as args
    vms_handle: HandleBase,
    self_handle: HandleBase,
    factory_handle: HandleBase,
}

impl Task {
    pub fn new_kernel() -> Arc<Task> {
        Arc::new(Self {
            inner: Spinlock::new(TaskInner::new_user()),
            name: "kernel task".into(),
            id: 0,
            vms: Vms::new_kernel(),
            handles: Spinlock::new(HandleTable::new()),
            factory: Factory::new(),
            vms_handle: HANDLE_INVALID,
            self_handle: HANDLE_INVALID,
            factory_handle: HANDLE_INVALID,
        })
    }

    pub fn new(name: String) -> Arc<Task> {
        let mut s = Arc::new(Self {
            inner: Spinlock::new(TaskInner::new_user()),
            name,
            id: 0,
            vms: Vms::new_user(),
            handles: Spinlock::new(HandleTable::new()),
            factory: Factory::new(),
            vms_handle: HANDLE_INVALID,
            self_handle: HANDLE_INVALID,
            factory_handle: HANDLE_INVALID,
        });

        let handle = Handle::new(s.clone());
        unsafe {
            Arc::get_mut_unchecked(&mut s).self_handle = handle.as_raw();
        }
        s.handle_table().add(handle);

        let handle = Handle::new(s.vms.clone());
        unsafe {
            Arc::get_mut_unchecked(&mut s).vms_handle = handle.as_raw();
        }
        s.handle_table().add(handle);

        let handle = Handle::new(s.factory.clone());
        unsafe {
            Arc::get_mut_unchecked(&mut s).factory_handle = handle.as_raw();
        }
        s.handle_table().add(handle);

        s
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn handle_table<'a>(&'a self) -> SpinlockGuard<'a, HandleTable> {
        self.handles.lock()
    }

    pub fn vms(&self) -> Arc<Vms> {
        self.vms.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_thread(&self, t: Arc<Thread>) {
        self.inner.lock().add_thread(t);
    }

    pub fn add_initial_thread(&self, t: Arc<Thread>, boot_handle: HandleBase) {
        t.setup_args(&[
            self.vms_handle,
            self.self_handle,
            self.factory_handle,
            boot_handle,
        ]);
        self.inner.lock().add_thread(t);
    }

    pub fn start_inner(&self) {
        self.inner.lock().start();
    }

    pub fn start(
        self: Arc<Self>,
        ep: VirtAddr,
        obj: Option<Arc<dyn KernelObject>>,
    ) -> Result<(), ErrorType> {
        use core::sync::atomic::{AtomicU16, Ordering};

        static ID_THREAD: AtomicU16 = AtomicU16::new(1);

        let init_thread = Thread::new(self.clone(), ID_THREAD.fetch_add(1, Ordering::Relaxed));
        let mut boot_handle: HandleBase = HANDLE_INVALID;
        let mut new_table = self.handle_table();

        if let Some(obj) = obj {
            let handle = Handle::new(obj);

            boot_handle = handle.as_raw();
            new_table.add(handle);
        }

        init_thread.init_user(ep);
        self.add_initial_thread(init_thread, boot_handle);

        self.start_inner();
        Ok(())
    }

    pub fn vms_handle(&self) -> HandleBase {
        self.vms_handle
    }
}
