use crate::{
    arch::{self, PAGE_SIZE},
    mm::{
        allocators::page_alloc::page_allocator,
        paging::page_table::{MappingType, PageTable},
        types::*,
        vma_list::{Vma, VmaList},
    },
};
use object_lib::object;

#[derive(object)]
pub struct Vms {
    start: VirtAddr,
    size: usize,
    ttbr0: Option<PageTable<false>>,
    vmas: VmaList,
}

impl Vms {
    pub fn new_kernel() -> VmsRef {
        Self::construct(Self {
            start: VirtAddr::from(0x0),
            size: usize::MAX,
            ttbr0: None,
            vmas: VmaList::new(),
        })
    }

    pub fn new_user() -> VmsRef {
        Self::construct(Self {
            start: VirtAddr::from(0x0),
            size: usize::MAX,
            ttbr0: Some(PageTable::new().unwrap()), // ToDo remove unwrap()
            vmas: VmaList::new(),
        })
    }

    pub fn add_vma_backed(&mut self, vma: Vma, backing: &[Pfn]) -> Option<()> {
        if let Some(ttbr0) = &mut self.ttbr0 {
            let mut va = vma.start();

            for i in backing {
                ttbr0
                    .map(
                        Some(MemRange::new(PhysAddr::from(*i), PAGE_SIZE)),
                        MemRange::new(va, PAGE_SIZE),
                        vma.map_flags(),
                    )
                    .ok()?;

                va.add(PAGE_SIZE);
            }

            self.vmas.add(vma);
            Some(())
        } else {
            None
        }
    }

    pub fn alloc_user_stack(&mut self) -> Option<VirtAddr> {
        let range = self
            .vmas
            .free_range(arch::PAGE_SIZE * 3, self.start, self.size)?;
        let pa = page_allocator().alloc(5)?;

        self.ttbr0.as_mut().unwrap()
            .map(
                Some(MemRange::new(
                    PhysAddr::from(pa + arch::PAGE_SIZE),
                    3 * arch::PAGE_SIZE,
                )),
                range,
                MappingType::UserData,
            )
            .ok()?;

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
