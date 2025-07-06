use super::vm_object::VmObject;
use crate::kernel::object::handle::Handle;
use crate::mm::paging::page_table::MmError;
use crate::mm::user_buffer::UserPtr;
use crate::mm::vms::VmsInner;
use alloc::sync::Arc;
use object_lib::object;
use qrwlock::RwLock;
use rtl::error::ErrorType;
use rtl::vmm::{types::*, MappingType};

#[derive(object)]
pub struct Vms {
    inner: RwLock<VmsInner>,
}

pub enum VmoCreateArgs {
    Backed(UserPtr<u8>, MappingType, VirtAddr),
    Zeroed(usize, MappingType, VirtAddr),
}

impl Vms {
    pub fn new_user() -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(VmsInner::new_user()),
        })
    }

    pub fn vm_map(
        &self,
        v: MemRange<VirtAddr>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, MmError> {
        let mut inner = self.inner.write();

        assert!(v.start().is_page_aligned());
        assert!(p.start().is_page_aligned());
        assert!(p.size().is_page_aligned());
        assert!(v.size().is_page_aligned());

        inner.vm_map(v, p, tp)
    }

    pub fn vm_allocate(&self, size: usize, tp: MappingType) -> Result<VirtAddr, ()> {
        let mut inner = self.inner.write();
        let res = inner.vm_allocate(size, tp)?;

        assert!(res.is_page_aligned());
        Ok(res)
    }

    pub fn vm_free(&self, base: VirtAddr, size: usize) -> Result<(), ErrorType> {
        let mut inner = self.inner.write();

        inner
            .vm_free(MemRange::new(base, size))
            .map_err(|_| ErrorType::INVALID_ARGUMENT)
    }

    pub fn base(&self) -> PhysAddr {
        let inner = self.inner.read();
        inner.ttbr0().unwrap()
    }

    pub fn create_vmo(&self, args: VmoCreateArgs) -> Result<Handle, ErrorType> {
        let vmo = match args {
            VmoCreateArgs::Backed(back, mt, ptr) => VmObject::from_buffer(back, mt, ptr),
            VmoCreateArgs::Zeroed(size, mt, ptr) => VmObject::zeroed(size, mt, ptr),
        }
        .ok_or(ErrorType::NO_MEMORY)?;

        Ok(Handle::new(vmo.clone()))
    }

    pub fn map_phys(&self, pa: PhysAddr, size: usize) -> Result<*mut u8, ErrorType> {
        let mut inner = self.inner.write();

        let va = inner
            .vm_map(
                MemRange::new(VirtAddr::new(0), size),
                MemRange::new(pa, size),
                MappingType::USER_DEVICE,
            )
            .unwrap();

        Ok(va.to_raw_mut::<u8>())
    }
}
