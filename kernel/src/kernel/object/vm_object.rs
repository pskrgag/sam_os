use crate::kernel::locking::spinlock::Spinlock;
use crate::mm::allocators::page_alloc::page_allocator;
use alloc::sync::Arc;
use hal::address::*;
use hal::arch::PAGE_SIZE;
use object_lib::object;
use rtl::vmm::MappingType;

#[derive(Debug)]
struct VmObjectInner {
    start: PhysAddr,
    pages: usize,
    mt: MappingType,
}

#[derive(object)]
pub struct VmObject {
    inner: Spinlock<VmObjectInner>,
}

impl VmObjectInner {
    pub fn zeroed(size: usize, tp: MappingType) -> Option<Self> {
        let pages = size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;
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
            inner: Spinlock::new(VmObjectInner::zeroed(size, tp)?),
        })
        .ok()
    }

    pub fn range(&self) -> MemRange<PhysAddr> {
        let inner = self.inner.lock();

        MemRange::new(inner.start, inner.pages * PAGE_SIZE)
    }

    pub fn mapping_type(&self) -> MappingType {
        let inner = self.inner.lock();

        inner.mt
    }
}

impl Drop for VmObjectInner {
    fn drop(&mut self) {
        page_allocator().free(self.start, self.pages);
    }
}

impl core::fmt::Debug for VmObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VmObject [ {:?} ]", *self.inner.lock()))
    }
}
