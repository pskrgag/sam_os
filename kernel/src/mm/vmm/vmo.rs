use crate::mm::paging::page_table::PageSource;
use crate::mm::pmm::page_alloc::page_allocator;
use crate::mm::pmm::page_list::{PageList, PageListIterator};
use crate::object::KernelObjectBase;
use alloc::sync::Arc;
use hal::address::{MemRange, PhysAddr};
use hal::arch::PAGE_SIZE;
use rtl::signal::Signal;
use rtl::vmm::MappingType;

enum VmPageBacking {
    List(PageList),
    Contig(MemRange<PhysAddr>),
}

pub enum VmObjectPagesIter<'a> {
    List(PageListIterator<'a>),
    Contig(MemRange<PhysAddr>),
}

impl PageSource for VmObjectPagesIter<'_> {
    fn next_page(&mut self) -> Option<PhysAddr> {
        match self {
            Self::List(list) => list.next_page(),
            Self::Contig(range) => range.next_page(),
        }
    }
}

struct VmObjectInner {
    source: VmPageBacking,
    mt: MappingType,
}

pub struct VmObject {
    inner: VmObjectInner,
    base: KernelObjectBase,
}

crate::kernel_object!(VmObject, Signal::None.into());

impl VmPageBacking {
    fn source<'a>(&'a self) -> VmObjectPagesIter<'a> {
        match self {
            Self::Contig(range) => VmObjectPagesIter::Contig(*range),
            Self::List(list) => VmObjectPagesIter::List(list.iter()),
        }
    }

    fn pages(&self) -> usize {
        match self {
            Self::Contig(range) => range.size() / PAGE_SIZE,
            Self::List(list) => list.pages(),
        }
    }
}

impl VmObjectInner {
    pub fn new(size: usize, mt: MappingType) -> Option<Self> {
        let pages = size.div_ceil(PAGE_SIZE);
        let list = page_allocator().alloc_pages(pages)?;
        let source = VmPageBacking::List(list);

        Some(Self { source, mt })
    }

    pub fn new_contig(size: usize, mt: MappingType) -> Option<Self> {
        let pages = size.div_ceil(PAGE_SIZE);
        let pa = page_allocator().alloc_contigious(pages)?;
        let source = VmPageBacking::Contig(MemRange::new(pa, pages));

        Some(Self { source, mt })
    }
}

impl VmObject {
    pub fn new(size: usize, tp: MappingType) -> Option<Arc<Self>> {
        Arc::try_new(Self {
            inner: VmObjectInner::new(size, tp)?,
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn new_contig(size: usize, tp: MappingType) -> Option<Arc<Self>> {
        Arc::try_new(Self {
            inner: VmObjectInner::new(size, tp)?,
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn size(&self) -> usize {
        self.inner.source.pages() * PAGE_SIZE
    }

    pub fn source(&self) -> impl PageSource {
        self.inner.source.source()
    }

    pub fn mapping_type(&self) -> MappingType {
        self.inner.mt
    }
}

impl Drop for VmObjectInner {
    fn drop(&mut self) {
        let mut old = PageList::default();

        match &mut self.source {
            VmPageBacking::List(list) => {
                core::mem::swap(list, &mut old);
                page_allocator().free(old);
            }
            VmPageBacking::Contig(range) => {
                page_allocator().free_contig(range.start(), range.size() / PAGE_SIZE);
            }
        }
    }
}
