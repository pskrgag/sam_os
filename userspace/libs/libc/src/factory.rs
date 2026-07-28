use crate::handle::Handle;
use crate::port::Port;
use crate::syscalls::Syscall;
use crate::task::Task;
use crate::vmm::vm_object::VmObject;
use alloc::string::ToString;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub static mut SELF_FACTORY: Option<Factory> = None;

pub struct Factory {
    h: Handle,
}

impl Factory {
    pub const fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn create_task(&self, name: &str) -> Result<Task, ErrorType> {
        Ok(Task::new(
            Syscall::create_task(&self.h, name)?,
            name.to_string(),
        ))
    }

    pub fn create_port(&self) -> Result<Port, ErrorType> {
        Syscall::create_port(&self.h).map(|x| unsafe { Port::new(x) })
    }

    pub fn create_vm_object(
        &self,
        size: usize,
        tp: MappingType,
    ) -> Result<VmObject, ErrorType> {
        let handle = Syscall::create_vmo(&self.h, size, tp)?;

        Ok(unsafe { VmObject::new(handle) })
    }

    pub fn create_vm_object_contig(
        &self,
        size: usize,
        tp: MappingType,
    ) -> Result<VmObject, ErrorType> {
        let handle = Syscall::create_vmo_contig(&self.h, size, tp)?;

        Ok(unsafe { VmObject::new(handle) })
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
