use crate::kernel::locking::spinlock::Spinlock;
use crate::mm::allocators::page_alloc::page_allocator;
use crate::mm::user_buffer::UserPtr;
use alloc::sync::Arc;
use object_lib::object;
use rtl::arch::{PAGE_SHIFT, PAGE_SIZE};
use rtl::error::ErrorType;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

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
    pub fn from_buffer(b: UserPtr<u8>, tp: MappingType, mut load_addr: VirtAddr) -> Option<Self> {
        let pages = ((load_addr.bits() + b.len()) >> PAGE_SHIFT)
            - ((load_addr.bits() as usize) >> PAGE_SHIFT)
            + 1;

        let p: PhysAddr = page_allocator().alloc(pages)?;
        let mut va = VirtAddr::from(p);

        unsafe { va.as_slice_mut::<u8>(pages * PAGE_SIZE).fill(0x00) };

        let range = unsafe { va.as_slice_at_offset_mut::<u8>(b.len(), load_addr.page_offset()) };

        let s = b.read_to(range);
        assert!(s == Some(b.len()));

        Some(Self {
            start: p,
            pages,
            mt: tp,
            load_addr: *load_addr.round_down_page(),
        })
    }

    pub fn zeroed(size: usize, tp: MappingType, mut load_addr: VirtAddr) -> Option<Self> {
        let pages = ((load_addr.bits() + size) >> PAGE_SHIFT)
            - ((load_addr.bits() as usize) >> PAGE_SHIFT)
            + 1;
        let p = page_allocator().alloc(pages)?;
        let mut va = VirtAddr::from(p);

        unsafe { va.as_slice_mut::<u8>(pages * PAGE_SIZE).fill(0x00) };

        Some(Self {
            start: p,
            pages,
            mt: tp,
            load_addr: *load_addr.round_down_page(),
        })
    }
}

impl VmObject {
    pub fn from_buffer(b: UserPtr<u8>, tp: MappingType, load_addr: VirtAddr) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            inner: Spinlock::new(VmObjectInner::from_buffer(b, tp, load_addr)?),
        }))
    }

    pub fn zeroed(size: usize, tp: MappingType, load_addr: VirtAddr) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            inner: Spinlock::new(VmObjectInner::zeroed(size, tp, load_addr)?),
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

    fn do_invoke(&self, _args: &[usize]) -> Result<usize, ErrorType> {
        todo!()
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
