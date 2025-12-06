use super::paging::kernel_page_table::kernel_page_table;
use crate::mm::{
    allocators::page_alloc::page_allocator,
    paging::page_table::PageTable,
    vma_list::{VmaFlag, VmaList},
};
use hal::address::*;
use hal::arch::*;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub struct VmsInner {
    ttbr0: Option<PageTable>,
    vmas: VmaList,
}

impl VmsInner {
    pub fn new_user() -> Option<Self> {
        Some(Self {
            ttbr0: Some(PageTable::new()?),
            vmas: VmaList::new_user(),
        })
    }

    pub fn new_kernel() -> Self {
        Self {
            ttbr0: None,
            vmas: VmaList::new_kernel(),
        }
    }

    pub fn vm_map(
        &mut self,
        v: Option<MemRange<VirtAddr>>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, ErrorType> {
        debug_assert!(p.start().is_page_aligned());
        debug_assert!(p.size().is_page_aligned());

        let size = p.size();

        let va = self.vmas.new_vma(
            size,
            v.map(|x| x.start()).map(|x| x.bits()),
            tp,
            VmaFlag::ExternalPages.into(),
        )?;

        self.ttbr0
            .as_mut()
            .unwrap()
            .map(p, MemRange::new(va, size), tp)?;

        Ok(va)
    }

    // ToDo: on-demang allocation of physical memory
    pub fn vm_allocate(&mut self, mut size: usize, tp: MappingType) -> Result<VirtAddr, ErrorType> {
        if !size.is_page_aligned() {
            return Err(ErrorType::InvalidArgument);
        }

        let mut new_va = self.vmas.new_vma(size, None, tp, VmaFlag::None.into())?;
        let ret = new_va;

        while size != 0 {
            let p = if let Some(p) = page_allocator().alloc(1) {
                p
            } else {
                return Err(ErrorType::NoMemory);
            };

            // ToDo: clean up in case of an error
            self.ttbr0
                .as_mut()
                .unwrap_or(&mut kernel_page_table())
                .map(
                    MemRange::new(p, PAGE_SIZE),
                    MemRange::new(new_va, PAGE_SIZE),
                    tp,
                )
                .map_err(|_| ErrorType::NoMemory)?;

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
            .unwrap_or(&mut kernel_page_table())
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
