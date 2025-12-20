use super::port_object::Port;
use crate::object::capabilities::CapabilityMask;
use crate::object::handle::Handle;
use crate::object::KernelObjectBase;
use crate::sched::current;
use crate::tasks::task::Task;
use alloc::string::ToString;
use alloc::sync::Arc;
use rtl::error::ErrorType;
use rtl::signal::Signal;
use spin::Lazy;

pub struct Factory {
    base: KernelObjectBase,
}

crate::kernel_object!(Factory, Signal::None.into());

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
        let task = current().task();
        let port = Port::new(task.clone()).ok_or(ErrorType::NoMemory)?;

        Ok(Handle::new(port, Port::full_caps()))
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        panic!("Factory dropped");
    }
}
