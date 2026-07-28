use crate::handle::Handle;
use crate::syscalls::Syscall;
use hal::address::PhysAddr;
use rtl::error::ErrorType;

pub struct VmObject {
    h: Handle,
}

impl VmObject {
    pub unsafe fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }

    pub fn get_phys_info(&self) -> Result<PhysAddr, ErrorType> {
        Syscall::vmo_get_phys_info(&self.h)
    }
}
