use crate::handle::Handle;
use crate::syscalls::{Syscall, VmoCreateArgs};
use crate::vmm::vm_object::VmObject;
use rtl::error::ErrorType;
use hal::address::*;
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

    pub fn create_vm_object(
        &self,
        b: &[u8],
        tp: MappingType,
        load_addr: VirtAddr,
    ) -> Result<VmObject, ErrorType> {
        let h: Handle = Syscall::vm_create_vmo(
            &self.h,
            VmoCreateArgs::Backed(b.as_ptr(), b.len(), tp, load_addr),
        )?;

        Ok(VmObject::new(h))
    }

    pub fn create_vm_object_zeroed(
        &self,
        tp: MappingType,
        load_addr: VirtAddr,
        size: usize,
    ) -> Result<VmObject, ErrorType> {
        let h: Handle =
            Syscall::vm_create_vmo(&self.h, VmoCreateArgs::Zeroed(size, tp, load_addr))?;

        Ok(VmObject::new(h))
    }

    pub fn map_vm_object(&self, o: &VmObject) -> Result<(), ErrorType> {
        Syscall::vm_map_vmo(&self.h, o.handle())
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
