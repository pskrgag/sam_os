use super::port_object::Port;
use super::task_object::Task;
use crate::kernel::object::capabilities::CapabilityMask;
use crate::kernel::object::handle::Handle;
use crate::kernel::object::KernelObjectBase;
use crate::kernel::sched::current;
use alloc::string::ToString;
use alloc::sync::Arc;
use object_lib::object;
use rtl::error::ErrorType;
use spin::Lazy;

#[derive(object)]
pub struct Factory {
    base: KernelObjectBase,
}

pub static FACTORY: Lazy<Arc<Factory>> = Lazy::new(|| Factory::new().unwrap());

impl Factory {
    fn new() -> Option<Arc<Factory>> {
        Some(
            Arc::try_new(Self {
                base: KernelObjectBase::new(),
            })
            .ok()?,
        )
    }

    pub fn create_task(&self, name: &str) -> Result<Handle, ErrorType> {
        let task = Task::new(name.to_string()).ok_or(ErrorType::NoMemory)?;
        let handle = Handle::new(task, CapabilityMask::any());

        Ok(handle)
    }

    pub fn create_port(&self) -> Result<Handle, ErrorType> {
        let task = current().unwrap().task();
        let port = Port::new(task.clone()).ok_or(ErrorType::NoMemory)?;

        Ok(Handle::new(port, Port::full_caps()))
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        panic!("Factory dropped");
    }
}
