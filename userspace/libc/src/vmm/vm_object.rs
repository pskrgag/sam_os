use rtl::handle::Handle;
use rtl::vmm::MappingType;
use rtl::vmm::types::VirtAddr;
use crate::syscalls::Syscall;

pub struct VmObject {
    h: Handle,
}

impl VmObject {
    pub fn new_from_buf(b: &[u8], tp: MappingType, load_addr: VirtAddr) -> Option<Self> {
        let h = Syscall::create_vm_object(b, tp, load_addr).ok()?;

        Some(Self {
            h
        })
    }

    pub fn handle(&self) -> Handle {
        self.h
    }
}
