use crate::{
    arch::{self, PAGE_SIZE},
    kernel::misc::num_pages,
    mm::{
        allocators::page_alloc::page_allocator,
        paging::kernel_page_table::kernel_page_table,
        paging::page_table::{MappingType, PageTable},
        types::*,
        vma_list::VmaList,
    },
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
            ttbr0: if !user {
                None
            } else {
                Some(PageTable::new(false)?)
            },
            vmas: VmaList::new(),
        })
    }

    pub fn add_vma(
        &mut self,
        data: (MemRange<VirtAddr>, MappingType),
        source: Option<&[u8]>,
    ) -> Option<()> {
        let num_pages = num_pages(data.0.size());
        let pa: PhysAddr = page_allocator().alloc(num_pages)?.into();
        let range = MemRange::new(*data.0.start().round_down_page(), data.0.size());


        if let Some(ttbr0) = &mut self.ttbr0 {
            ttbr0
                .map(
                    Some(MemRange::new(pa.into(), PAGE_SIZE * num_pages)),
                    range.clone(),
                    data.1,
                )
                .ok()?;

            // TODO: bad solution
            if let Some(s) = source {
                kernel_page_table()
                    .map(
                        None,
                        MemRange::new(VirtAddr::from(pa), num_pages * PAGE_SIZE),
                        MappingType::KernelData,
                    )
                    .ok()?;

                #[allow(unused_unsafe)]
                unsafe {
                    core::slice::from_raw_parts_mut(
                        VirtAddr::from(pa)
                            .to_raw_mut::<u8>()
                            .add(data.0.start().page_offset()),
                        s.len(),
                    )
                    .copy_from_slice(s);
                }
            }

            self.vmas.add(data.0, data.1);
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
