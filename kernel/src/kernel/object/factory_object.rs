use super::task_object::Task;
use super::port_object::Port;
use super::handle::Handle;
use crate::kernel::sched::current;
use alloc::string::ToString;
use object_lib::object;
use rtl::error::ErrorType;
use rtl::objects::factory::FactroryInvoke;
use alloc::sync::Arc;
use crate::mm::user_buffer::UserBuffer;

#[derive(object)]
pub struct Factory {
    // ??
}

impl Factory {
    pub fn new() -> Arc<Factory> {
        Arc::new(Self {})
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        match FactroryInvoke::from_bits(args[0]).ok_or(ErrorType::NO_OPERATION)? {
            FactroryInvoke::CREATE_TASK => {
                let name = UserBuffer::new(args[1].into(), args[2]);
                let name = name.read_on_stack::<100>().ok_or(ErrorType::FAULT)?;
                let name = core::str::from_utf8(&name).map_err(|_| ErrorType::INVALID_ARGUMENT)?;
                let new_task = Task::new(name.to_string());

                let task = current().unwrap().task();
                let mut table = task.handle_table();

                let handle = Handle::new(new_task.clone());
                let ret = handle.as_raw();

                table.add(handle);

                Ok(ret)
            }
            FactroryInvoke::CREATE_PORT => {
                let thread = current().unwrap();
                let task = thread.task();
                let mut table = task.handle_table();

                let port = Port::new(thread);
                let handle = Handle::new(port.clone());
                let ret = handle.as_raw();

                table.add(handle);

                Ok(ret)
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
