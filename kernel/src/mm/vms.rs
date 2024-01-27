use crate::mm::{
    allocators::page_alloc::page_allocator,
    paging::page_table::{MmError, PageTable},
    vma_list::{Vma, VmaList},
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

    pub fn free_range(&self, size: usize) -> Option<MemRange<VirtAddr>> {
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
        let mut range = if let Some(r) = self.free_range(size) {
            r
        } else {
            return Err(());
        };
        let va = range.start();

        assert!(size.is_page_aligned());

        self.add_to_tree(Vma::new(range, tp))?;

        while range.size() != 0 {
            let p: PhysAddr = if let Some(p) = page_allocator().alloc(1) {
                p.into()
            } else {
                return Err(());
            };

            println!("Mapping {:?}", range);
            // ToDo: clean up in case of an error
            self.ttbr0.as_mut().unwrap().map(
                Some(MemRange::new(p, PAGE_SIZE)),
                MemRange::new(range.start().into(), PAGE_SIZE),
                tp,
            )?;

            range = MemRange::new(
                VirtAddr::from(range.start() + PAGE_SIZE),
                range.size() - PAGE_SIZE,
            );
        }

        Ok(va)
    }

    pub fn vm_free(&mut self, range: MemRange<VirtAddr>) -> Result<(), ()> {
        assert!(range.start().is_page_aligned());
        assert!(range.size().is_page_aligned());

        self.vmas.mark_free(Vma::new(range, MappingType::USER_DATA));

        self.ttbr0.as_mut().unwrap().free(range, |pa, device| {
            if !device {
                page_allocator().free(pa, 1);
            }
        })?;

        Ok(())
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        if let Some(ttbr0) = &self.ttbr0 {
            Some(ttbr0.base())
        } else {
            None
        }
    }
}
