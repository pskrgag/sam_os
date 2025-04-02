use crate::mm::paging::kernel_page_table::kernel_page_table;

use crate::arch::KERNEL_MMIO_BASE;
use crate::kernel::locking::spinlock::Spinlock;
use rtl::arch::PAGE_SIZE;
use rtl::misc::num_pages;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

extern "C" {
    static mmio_start: usize;
    static mmio_end: usize;
}

/* I don't see why couple of kernel drivers would like to unmap their device */
pub struct MmioAllocator {
    start: VirtAddr,
    pages: usize,
    offset: usize,
}

pub static MMIO_ALLOCATOR: Spinlock<MmioAllocator> = Spinlock::new(MmioAllocator::default());

impl MmioAllocator {
    pub const fn default() -> Self {
        Self {
            start: VirtAddr::new(0),
            pages: 0,
            offset: 0,
        }
    }

    pub fn new() -> Self {
        Self {
            start: KERNEL_MMIO_BASE.into(),
            pages: num_pages(linker_var!(mmio_end) - linker_var!(mmio_start)),
            offset: 0,
        }
    }

    pub fn iomap(&mut self, addr: PhysAddr, pages: usize) -> Option<VirtAddr> {
        if self.pages < pages {
            return None;
        }

        let new_va = VirtAddr::new(self.start + self.offset * PAGE_SIZE);

        kernel_page_table()
            .map(
                Some(MemRange::new(addr, PAGE_SIZE * pages)),
                MemRange::new(new_va, PAGE_SIZE * pages),
                MappingType::KERNEL_DEVICE,
            )
            .ok()?;

        self.offset += pages;
        self.pages -= pages;
        Some(new_va)
    }

    pub fn free_pages(&self) -> usize {
        self.pages
    }
}

pub fn init() {
    let new_allocator = MmioAllocator::new();

    *MMIO_ALLOCATOR.lock() = new_allocator;
}
