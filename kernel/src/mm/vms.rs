use crate::mm::{
    allocators::page_alloc::page_allocator,
    layout::vmm_range,
    paging::page_table::{MmError, PageTable},
    vma_list::{MemRangeVma, Vma, VmaList},
};
use loader_protocol::VmmLayoutKind;
use rtl::arch::*;
use rtl::error::ErrorType;
use rtl::vmm::{MappingType, types::*};

pub struct VmsInner {
    size: usize,
    start: VirtAddr,
    ttbr0: Option<PageTable>,
    vmas: VmaList,
}

impl VmsInner {
    pub fn new_user() -> Self {
        Self {
            start: VirtAddr::from(0x0),
            size: usize::MAX,
            ttbr0: Some(PageTable::new().unwrap()), // ToDo remove unwrap()
            vmas: VmaList::new(),
        }
    }

    pub fn new_kernel() -> Self {
        Self {
            start: VirtAddr::from(vmm_range(VmmLayoutKind::VmAlloc).start()),
            size: usize::MAX,
            ttbr0: None,
            vmas: VmaList::new(),
        }
    }

    fn add_to_tree(&mut self, vma: Vma) -> Result<VirtAddr, MmError> {
        self.vmas.add_to_tree(vma).map_err(|_| MmError::Generic)
    }

    pub fn vm_map(
        &mut self,
        v: MemRange<VirtAddr>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, MmError> {
        assert!(v.start().is_page_aligned());
        assert!(p.start().is_page_aligned());

        let va = self.add_to_tree(Vma::new(v.into(), tp))?;

        self.ttbr0
            .as_mut()
            .unwrap()
            .map(Some(p), MemRange::new(va, v.size()), tp)?;

        Ok(va)
    }

    // ToDo: on-demang allocation of physical memory
    pub fn vm_allocate(&mut self, mut size: usize, tp: MappingType) -> Result<VirtAddr, ()> {
        if !size.is_page_aligned() {
            return Err(());
        }

        let mut new_va = self.add_to_tree(Vma::new(MemRangeVma::new_non_fixed(size), tp))?;
        let ret = new_va;

        while size != 0 {
            let p = if let Some(p) = page_allocator().alloc(1) {
                p
            } else {
                return Err(());
            };

            // ToDo: clean up in case of an error
            self.ttbr0.as_mut().unwrap().map(
                Some(MemRange::new(p, PAGE_SIZE)),
                MemRange::new(new_va, PAGE_SIZE),
                tp,
            )?;

            size -= PAGE_SIZE;
            new_va.add(PAGE_SIZE);
        }

        Ok(ret)
    }

    pub fn vm_free(&mut self, range: MemRange<VirtAddr>) -> Result<(), ErrorType> {
        assert!(range.start().is_page_aligned());
        assert!(range.size().is_page_aligned());

        self.vmas.free(range)?;

        self.ttbr0
            .as_mut()
            .unwrap()
            .free(range, |pa, device| {
                if !device {
                    page_allocator().free(pa, 1);
                }
            })
            .expect("Failed to free memory");

        Ok(())
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        self.ttbr0.as_ref().map(|ttbr0| ttbr0.base())
    }
}
