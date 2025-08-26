use crate::handle::Handle;
use crate::port::Port;
use crate::syscalls::Syscall;
use crate::task::Task;
use alloc::string::ToString;

pub static mut SELF_FACTORY: Option<Factory> = None;

pub struct Factory {
    h: Handle,
}

impl Factory {
    pub const fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn create_task(&self, name: &str) -> Option<Task> {
        Some(Task::new(
            Syscall::create_task(&self.h, name).ok()?,
            name.to_string(),
        ))
    }

    pub fn create_port(&self) -> Option<Port> {
        Some(Port::new(Syscall::create_port(&self.h).ok()?))
    }
}

unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

pub fn init_self_factory(h: Handle) {
    unsafe {
        SELF_FACTORY = Some(Factory::new(h));
    }
}

#[allow(static_mut_refs)]
pub fn factory() -> &'static Factory {
    unsafe { SELF_FACTORY.as_ref().unwrap() }
}
