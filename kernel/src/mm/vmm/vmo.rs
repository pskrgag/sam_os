use crate::mm::pmm::page_alloc::page_allocator;
use crate::mm::pmm::page_list::PageList;
use crate::object::KernelObjectBase;
use alloc::sync::Arc;
use hal::arch::PAGE_SIZE;
use rtl::signal::Signal;
use rtl::vmm::MappingType;

struct VmObjectInner {
    list: PageList,
    mt: MappingType,
}

pub struct VmObject {
    inner: VmObjectInner,
    base: KernelObjectBase,
}

crate::kernel_object!(VmObject, Signal::None.into());

impl VmObjectInner {
    pub fn zeroed(size: usize, mt: MappingType) -> Option<Self> {
        let pages = size.div_ceil(PAGE_SIZE);
        let list = page_allocator().alloc_pages(pages)?;

        Some(Self { list, mt })
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

    pub fn size(&self) -> usize {
        self.list().pages() * PAGE_SIZE
    }

    pub fn list(&self) -> &PageList {
        let inner = &self.inner;

        &inner.list
    }

    pub fn mapping_type(&self) -> MappingType {
        self.inner.mt
    }
}

impl Drop for VmObjectInner {
    fn drop(&mut self) {
        let mut old = PageList::default();

        core::mem::swap(&mut self.list, &mut old);
        page_allocator().free(old);
    }
}
