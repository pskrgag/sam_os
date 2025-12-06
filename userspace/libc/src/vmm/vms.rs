use crate::handle::Handle;
use crate::syscalls::Syscall;
use crate::vmm::vm_object::VmObject;
use hal::address::*;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub static mut SELF_VMS: Option<Vms> = None;

pub struct Vms {
    h: Handle,
}

impl Vms {
    pub const fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn vm_allocate(&self, size: usize, mt: MappingType) -> Result<*mut u8, ErrorType> {
        Syscall::vm_allocate(&self.h, size, mt)
    }

    pub fn vm_free(&self, addr: *mut u8, size: usize) -> Result<(), ErrorType> {
        Syscall::vm_free(&self.h, addr, size)
    }

    pub fn create_vm_object(&self, size: usize, tp: MappingType) -> Result<VmObject, ErrorType> {
        let h: Handle = Syscall::vm_create_vmo(&self.h, size, tp)?;

        Ok(VmObject::new(h))
    }

    pub fn map_vm_object(&self, o: &VmObject, to: Option<VirtAddr>, tp: MappingType) -> Result<VirtAddr, ErrorType> {
        Syscall::vm_map_vmo(&self.h, o.handle(), to.unwrap_or(VirtAddr::new(0)), tp)
    }

    pub fn map_phys(&self, p: MemRange<PhysAddr>) -> Option<VirtAddr> {
        Syscall::vm_map_phys(&self.h, p.start(), p.size())
            .ok()
            .map(VirtAddr::from)
    }
}

unsafe impl Send for Vms {}
unsafe impl Sync for Vms {}

pub fn init_self_vms(h: Handle) {
    unsafe {
        SELF_VMS = Some(Vms::new(h));
    }
}

#[allow(static_mut_refs)]
pub fn vms() -> &'static Vms {
    unsafe { SELF_VMS.as_ref().unwrap() }
}
