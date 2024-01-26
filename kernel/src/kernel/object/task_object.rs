use crate::kernel::locking::spinlock::*;
use crate::kernel::object::factory_object::Factory;
use crate::kernel::object::handle::Handle;
use crate::kernel::object::handle_table::HandleTable;
use crate::kernel::object::thread_object::Thread;
use crate::kernel::object::vms_object::Vms;
use crate::kernel::tasks::task::TaskInner;
use crate::sched::current;
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

    pub fn handle_table(&self) -> SpinlockGuard<HandleTable> {
        self.handles.lock()
    }

    pub fn vms(&self) -> Arc<Vms> {
        self.vms.clone()
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

    pub fn start(&self) {
        self.inner.lock().start()
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        use rtl::objects::task::TaskInvoke;
        use core::sync::atomic::{AtomicU16, Ordering};

        static ID_THREAD: AtomicU16 = AtomicU16::new(1);

        match TaskInvoke::from_bits(args[0]).ok_or(ErrorType::NO_OPERATION)? {
            TaskInvoke::START => {
                let ep: VirtAddr = args[1].into();
                let h: HandleBase = args[2].into();
                let curr = current().unwrap().task();
                let cur_table = curr.handle_table();
                let mut new_table = self.handle_table();

                // ToDo: this is ugly as fuck
                let s = cur_table.find::<Task>(self.self_handle).unwrap();
                let init_thread = Thread::new(s.clone(), ID_THREAD.fetch_add(1, Ordering::Relaxed));

                let mut boot_handle: HandleBase = HANDLE_INVALID;

                if h != HANDLE_INVALID {
                    let obj = cur_table.find_poly(h).ok_or(ErrorType::INVALID_HANDLE)?;
                    let handle = Handle::new(obj);

                    boot_handle = handle.as_raw();
                    new_table.add(handle);
                }

                init_thread.init_user(ep);
                self.add_initial_thread(init_thread, boot_handle);

                self.start();
                Ok(0)
            }
            TaskInvoke::GET_VMS => {
                let task = current().unwrap().task();
                let mut table = task.handle_table();

                let handle = Handle::new(self.vms.clone());
                let ret = handle.as_raw();
                table.add(handle);

                Ok(ret)
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
