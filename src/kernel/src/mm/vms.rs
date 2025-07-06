use crate::mm::{
    allocators::page_alloc::page_allocator,
    paging::page_table::{MmError, PageTable},
    vma_list::{MemRangeVma, Vma, VmaList},
};
use rtl::arch::*;
use rtl::vmm::{types::*, MappingType};

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

    fn add_to_tree(&mut self, vma: Vma) -> Result<VirtAddr, ()> {
        self.vmas.add_to_tree(vma)
    }

    pub fn vm_map(
        &mut self,
        v: MemRange<VirtAddr>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, MmError> {
        let va = self.add_to_tree(Vma::new(v.into(), tp))?;

        self.ttbr0
            .as_mut()
            .unwrap()
            .map(Some(p), MemRange::new(va, v.size()), tp)?;

        assert!(v.start().is_page_aligned());
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
            let mut va = VirtAddr::from(p);

            unsafe { va.as_slice_mut::<u8>(PAGE_SIZE).fill(0x00) };

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

    pub fn vm_free(&mut self, range: MemRange<VirtAddr>) -> Result<(), ()> {
        assert!(range.start().is_page_aligned());
        assert!(range.size().is_page_aligned());

        self.vmas
            .mark_free(Vma::new(range.into(), MappingType::USER_DATA))
            .ok_or(())?;

        self.ttbr0.as_mut().unwrap().free(range, |pa, device| {
            if !device {
                page_allocator().free(pa, 1);
            }
        })?;

        Ok(())
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        self.ttbr0.as_ref().map(|ttbr0| ttbr0.base())
    }
}
