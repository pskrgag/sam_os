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

    pub fn create_task(&self, name: &str) -> Result<Arc<Task>, ErrorType> {
        Ok(Task::new(name.to_string()))
    }

    pub fn create_port(&self) -> Result<Arc<Port>, ErrorType> {
        let task = current().unwrap().task();

        Ok(Port::new(task.clone()))
    }
}
