use crate::kernel::locking::spinlock::Spinlock;
use crate::kernel::misc::num_pages;
use crate::mm::allocators::page_alloc::page_allocator;
use object_lib::object;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;
use rtl::arch::PAGE_SIZE;

#[derive(Debug)]
struct VmObjectInner {
    start: PhysAddr,
    pages: usize,
    mt: MappingType,
    load_addr: VirtAddr,
}

#[derive(object)]
pub struct VmObject {
    inner: Spinlock<VmObjectInner>,
}

impl VmObjectInner {
    pub fn from_buffer(b: &[u8], tp: MappingType, mut load_addr: VirtAddr) -> Option<Self> {
        let pages = num_pages(b.len());

        let p: PhysAddr = page_allocator().alloc(pages)?;
        let va = VirtAddr::from(p);
        let range = unsafe { va.as_slice_mut::<u8>(b.len()) };

        range.copy_from_slice(b);

        Some(Self {
            start: p,
            pages,
            mt: tp,
            load_addr: *load_addr.round_down_page(),
        })
    }
}

impl VmObject {
    pub fn from_buffer(b: &[u8], tp: MappingType, load_addr: VirtAddr) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            inner: Spinlock::new(VmObjectInner::from_buffer(b, tp, load_addr)?),
        }))
    }

    pub fn as_ranges(&self) -> (MemRange<VirtAddr>, MemRange<PhysAddr>) {
        let inner = self.inner.lock();

        (
            MemRange::new(inner.load_addr, inner.pages * PAGE_SIZE),
            MemRange::new(inner.start, inner.pages * PAGE_SIZE),
        )
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
