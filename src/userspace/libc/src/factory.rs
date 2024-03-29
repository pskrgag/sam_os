use crate::port::Port;
use crate::syscalls::Syscall;
use crate::task::Task;
use alloc::string::ToString;
use rtl::handle::Handle;
use rtl::objects::factory::FactroryInvoke;

pub static mut SELF_FACTORY: Factory = Factory::new(0);

pub struct Factory {
    h: Handle,
}

impl Factory {
    pub const fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn create_task(&self, name: &str) -> Option<Task> {
        Some(Task::new(
            Syscall::invoke(
                self.h,
                FactroryInvoke::CREATE_TASK.bits(),
                &[name.as_ptr() as usize, name.len()],
            )
            .ok()?,
            name.to_string(),
        ))
    }

    pub fn create_port(&self) -> Option<Port> {
        Some(Port::new(
            Syscall::invoke(self.h, FactroryInvoke::CREATE_PORT.bits(), &[]).ok()?,
        ))
    }
}

unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

pub fn init_self_factory(h: Handle) {
    unsafe {
        SELF_FACTORY = Factory::new(h);
    }
}

#[allow(static_mut_ref)]
pub fn factory() -> &'static Factory {
    unsafe { &SELF_FACTORY }
}
