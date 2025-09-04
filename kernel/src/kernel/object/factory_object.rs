use super::handle::Handle;
use super::port_object::Port;
use super::task_object::Task;
use crate::kernel::sched::current;
use alloc::string::ToString;
use alloc::sync::Arc;
use object_lib::object;
use rtl::error::ErrorType;

#[derive(object)]
pub struct Factory {
    // ??
}

impl Factory {
    pub fn new() -> Arc<Factory> {
        Arc::new(Self {})
    }

    pub fn create_task(&self, name: &str) -> Result<Handle, ErrorType> {
        let new_task = Task::new(name.to_string());

        Ok(Handle::new(new_task.clone()))
    }

    pub fn create_port(&self) -> Result<Handle, ErrorType> {
        let task = current().unwrap().task();
        let port = Port::new(task.clone());

        Ok(Handle::new(port.clone()))
    }
}
