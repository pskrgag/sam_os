use super::task_object::Task;
use super::thread_object::Thread;
use super::handle::Handle;
use crate::kernel::sched::current;
use alloc::string::ToString;
use object_lib::object;
use rtl::error::ErrorType;
use rtl::objects::factory::FactroryInvoke;

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
                let name_range = unsafe {
                    core::slice::from_raw_parts(args[1] as *const u8, args[2])
                };
                let name = core::str::from_utf8(name_range).map_err(|_| ErrorType::FAULT)?;
                let new_task = Task::new(name.to_string());

                let task = current().unwrap().task();
                let mut table = task.handle_table();

                let handle = Handle::new::<Task>(new_task.clone());
                let ret = handle.as_raw();

                table.add(handle);

                Ok(ret)
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
