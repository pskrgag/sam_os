use super::vm_object::VmObject;
use crate::arch::mm::page_table::switch_context;
use crate::kernel::locking::mutex::Mutex;
use crate::kernel::object::capabilities::{Capability, CapabilityMask};
use crate::kernel::object::handle::Handle;
use crate::kernel::object::KernelObjectBase;
use crate::mm::vms::VmsInner;
use alloc::sync::Arc;
use hal::address::{Address, MemRange, PhysAddr, VirtAddr};
use object_lib::object;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

#[derive(object)]
pub struct Vms {
    inner: Mutex<VmsInner>,
    base: KernelObjectBase,
}

impl Vms {
    pub fn new_user() -> Option<Arc<Self>> {
        Arc::try_new(Self {
            inner: Mutex::new(VmsInner::new_user()?),
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn new_kernel() -> Option<Arc<Self>> {
        Arc::try_new(Self {
            inner: Mutex::new(VmsInner::new_kernel()),
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn full_caps() -> CapabilityMask {
        CapabilityMask::from(Capability::MapPhys)
    }

    pub fn vm_map(
        &self,
        v: Option<MemRange<VirtAddr>>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, ErrorType> {
        let mut inner = self.inner.lock();

        debug_assert!(p.start().is_page_aligned());
        debug_assert!(p.size().is_page_aligned());

        inner.vm_map(v, p, tp)
    }

    pub fn vm_allocate(&self, size: usize, tp: MappingType) -> Result<VirtAddr, ErrorType> {
        let mut inner = self.inner.lock();
        let res = inner.vm_allocate(size, tp)?;

        debug_assert!(res.is_page_aligned());
        Ok(res)
    }

    pub fn vm_free(&self, base: VirtAddr, size: usize) -> Result<(), ErrorType> {
        let mut inner = self.inner.lock();

        inner
            .vm_free(MemRange::new(base, size))
            .map_err(|_| ErrorType::InvalidArgument)
    }

    pub fn base(&self) -> PhysAddr {
        let inner = self.inner.lock();

        inner.ttbr0().unwrap()
    }

    pub fn create_vmo(&self, size: usize, mt: MappingType) -> Result<Handle, ErrorType> {
        let vmo = VmObject::zeroed(size, mt).ok_or(ErrorType::NoMemory)?;

        Ok(Handle::new(vmo, CapabilityMask::any()))
    }

    pub fn map_phys(&self, pa: PhysAddr, size: usize) -> Result<*mut u8, ErrorType> {
        let mut inner = self.inner.lock();

        let va = inner.vm_map(None, MemRange::new(pa, size), MappingType::Device)?;
        Ok(va.to_raw_mut::<u8>())
    }

    pub fn switch_to(&self) {
        switch_context(self.base());
    }

    pub fn translate(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.inner.lock().translate(va)
    }
}
