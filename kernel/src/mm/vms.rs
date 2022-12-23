use crate::{
    kernel::misc::num_pages,
    mm::{
        allocators::page_alloc::PAGE_ALLOC,
        paging::page_table::{MappingType, PageTable},
        types::*,
        vma_list::VmaList,
    },
    arch,
};

pub struct Vms {
    start: VirtAddr,
    size: usize,
    ttbr0: Option<PageTable>,
    vmas: VmaList,
}

impl Vms {
    pub fn new(start: VirtAddr, size: usize, user: bool) -> Option<Self> {
        Some(Self {
            start: start,
            size: size,
            ttbr0: if !user { None } else { Some(PageTable::new(false)?) },
            vmas: VmaList::new(),
        })
    }

    pub fn add_vma(&mut self, data: (MemRange<VirtAddr>, MappingType)) -> Option<()> {
        let num_pages = num_pages(data.0.size());
        let pa = PAGE_ALLOC.lock().alloc_pages(num_pages)?;

        if let Some(ttbr0) = &mut self.ttbr0 {
            ttbr0
                .map(Some(MemRange::new(pa, num_pages)), data.0.clone(), data.1)
                .ok()?;

            self.vmas.add(data.0, data.1);
            Some(())
        } else {
            None
        }
    }

    pub fn alloc_user_stack(&mut self) -> Option<VirtAddr> {
        let range = self.vmas.free_range(arch::PAGE_SIZE * 3, self.start, self.size)?;
        let pa = PAGE_ALLOC.lock().alloc_pages(5)?;

        if let Some(ttbr0) = &mut self.ttbr0 {
            ttbr0.map(Some(MemRange::new(pa + PhysAddr::from(arch::PAGE_SIZE), 3 * arch::PAGE_SIZE)), range, MappingType::UserData).ok()?;
        } else {
            panic!();
        }

        Some(range.start())
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        if let Some(ttbr0) = &self.ttbr0 {
            Some(ttbr0.base())
        } else {
            None
        }
    }
}

impl Default for Vms {
    fn default() -> Self {
        Self::new(VirtAddr::new(0), 0, false).unwrap()
    }
}

