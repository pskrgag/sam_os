use crate::syscalls::Syscall;
use crate::vmm::vm_object::VmObject;
use rtl::error::ErrorType;
use rtl::handle::Handle;
use rtl::objects::vms::VmsInvoke;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

pub static mut SELF_VMS: Vms = Vms::new(0);

pub struct Vms {
    h: Handle,
}

impl Vms {
    pub const fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn vm_allocate(&self, size: usize, mt: MappingType) -> Result<*mut u8, ErrorType> {
        Ok(Syscall::invoke(self.h, VmsInvoke::ALLOCATE.bits(), &[size, mt.bits()])? as *mut u8)
    }

    pub fn create_vm_object(
        &self,
        b: &[u8],
        tp: MappingType,
        load_addr: VirtAddr,
    ) -> Option<VmObject> {
        let h: Handle = Syscall::invoke(
            self.h,
            VmsInvoke::CREATE_VMO.bits(),
            &[b.as_ptr() as usize, b.len(), tp.into(), load_addr.into()],
        )
        .ok()?
        .into();

        Some(VmObject::new(h))
    }

    pub fn map_vm_object(&self, o: &VmObject) -> Option<()> {
        Syscall::invoke(self.h, VmsInvoke::MAP_VMO.bits(), &[o.handle()]).ok()?;
        Some(())
    }
}

unsafe impl Send for Vms {}
unsafe impl Sync for Vms {}

pub fn init_self_vms(h: Handle) {
    unsafe {
        SELF_VMS = Vms::new(h);
    }
}

pub fn vms() -> &'static Vms {
    unsafe { &SELF_VMS }
}
