use crate::{
    arch::{self, PAGE_SIZE},
    mm::{
        allocators::page_alloc::page_allocator,
        paging::page_table::{MappingType, PageTable},
        types::*,
        vma_list::{VmaList, Vma},
    },
};

pub struct Vms {
    start: VirtAddr,
    size: usize,
    ttbr0: Option<PageTable<false>>,
    vmas: VmaList,
}

impl Vms {
    pub fn new(start: VirtAddr, size: usize, user: bool) -> Option<Self> {
        Some(Self {
            start: start,

            size: size,
            ttbr0: if !user { None } else { Some(PageTable::new()?) },
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

        if let Some(ttbr0) = &mut self.ttbr0 {
            ttbr0
                .map(
                    Some(MemRange::new(
                        PhysAddr::from(pa + arch::PAGE_SIZE),
                        3 * arch::PAGE_SIZE,
                    )),
                    range,
                    MappingType::UserData,
                )
                .ok()?;
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
