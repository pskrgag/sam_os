use crate::{
    arch::{self, PAGE_SIZE},
    mm::{
        allocators::page_alloc::page_allocator,
        paging::page_table::{MappingType, PageTable, MmError},
        types::*,
        vma_list::{Vma, VmaList},
    },
};
use object_lib::object;

pub struct VmsInner {
    size: usize,
    start: VirtAddr,
    ttbr0: Option<PageTable<false>>,
    vmas: VmaList,
}

#[derive(object)]
pub struct Vms {
    inner: RwLock<VmsInner>,
}

impl Vms {
    pub fn new_kernel() -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(VmsInner::new_kernel()),
        })
    }

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
        inner.vm_map(v, p, tp)
    }

    pub fn vm_allocate(&self, size: usize, tp: MappingType) -> Result<VirtAddr, ()> {
        let mut inner = self.inner.write();
        inner.vm_allocate(size, tp)
    }

    pub fn base(&self) -> PhysAddr {
        let inner = self.inner.read();
        inner.ttbr0().unwrap()
    }
}

impl VmsInner {
    pub fn new_kernel() -> Self {
        Self {
            start: VirtAddr::from(0x0),
            size: usize::MAX,
            ttbr0: None,
            vmas: VmaList::new(),
        }
    }

    pub fn new_user() -> Self {
        Self {
            start: VirtAddr::from(0x0),
            size: usize::MAX,
            ttbr0: Some(PageTable::new().unwrap()), // ToDo remove unwrap()
            vmas: VmaList::new(),
        }
    }

    fn free_range(&self, size: usize) -> Option<MemRange<VirtAddr>> {
        self.vmas.free_range(size)
    }

    fn free_range_at(&self, range: MemRange<VirtAddr>) -> Option<MemRange<VirtAddr>> {
        self.vmas.free_range_at(range)
    }

    fn add_to_tree(&mut self, vma: Vma) -> Result<VirtAddr, ()> {
        self.vmas.add_to_tree(vma)
    }

    pub fn vm_map(
        &mut self,
        v: MemRange<VirtAddr>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, MmError> {
        let range = self.free_range_at(v).unwrap();

        self.add_to_tree(Vma::new(range, tp))?;

        self.ttbr0.as_mut().unwrap().map(Some(p), range, tp)?;
        assert!(v.start().is_page_aligned());
        Ok(v.start())
    }

    // ToDo: on-demang allocation of physical memory
    pub fn vm_allocate(&mut self, size: usize, tp: MappingType) -> Result<VirtAddr, ()> {
        let range = if let Some(r) = self.free_range(size) {
            r
        } else {
            return Err(());
        };
        let va = range.start();

        assert!(size % arch::PAGE_SIZE == 0);

        let p: PhysAddr = if let Some(p) = page_allocator().alloc(size >> arch::PAGE_SHIFT) {
            p.into()
        } else {
            return Err(());
        };

        self.add_to_tree(Vma::new(range, tp))?;

        // ToDo: clean up in case of an error
        self.ttbr0
            .as_mut()
            .unwrap()
            .map(Some(MemRange::new(p, size)), range, tp)?;

        Ok(va)
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        if let Some(ttbr0) = &self.ttbr0 {
            Some(ttbr0.base())
        } else {
            None
        }
    }
}
