use crate::mm::allocators::page_alloc::page_allocator;
use crate::object::KernelObjectBase;
use alloc::sync::Arc;
use hal::address::*;
use hal::arch::PAGE_SIZE;
use rtl::signal::Signal;
use rtl::vmm::MappingType;

#[derive(Debug)]
struct VmObjectInner {
    start: PhysAddr,
    pages: usize,
    mt: MappingType,
}

pub struct VmObject {
    inner: VmObjectInner,
    base: KernelObjectBase,
}

crate::kernel_object!(VmObject, Signal::None.into());

impl VmObjectInner {
    pub fn zeroed(size: usize, tp: MappingType) -> Option<Self> {
        let pages = size.div_ceil(PAGE_SIZE);
        let p = page_allocator().alloc(pages)?;

        Some(Self {
            start: p,
            pages,
            mt: tp,
        })
    }
}

impl VmObject {
    pub fn zeroed(size: usize, tp: MappingType) -> Option<Arc<Self>> {
        Arc::try_new(Self {
            inner: VmObjectInner::zeroed(size, tp)?,
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn range(&self) -> MemRange<PhysAddr> {
        let inner = &self.inner;

        MemRange::new(inner.start, inner.pages * PAGE_SIZE)
    }

    pub fn mapping_type(&self) -> MappingType {
        self.inner.mt
    }
}

impl Drop for VmObjectInner {
    fn drop(&mut self) {
        page_allocator().free(self.start, self.pages);
    }
}

impl core::fmt::Debug for VmObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VmObject [ {:?} ]", self.inner))
    }
}
