use crate::{
    arch,
    kernel::{locking::fake_lock::FakeLock, misc::num_pages},
    linker_var,
    mm::paging::{kernel_page_table::kernel_page_table, page_table::MappingType},
    mm::types::*,
    print, println,
};

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

pub static MMIO_ALLOCATOR: FakeLock<MmioAllocator> = FakeLock::new(MmioAllocator::default());

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
            start: VirtAddr::new(linker_var!(mmio_start)),
            pages: num_pages(linker_var!(mmio_end) - linker_var!(mmio_start)),
            offset: 0,
        }
    }

    pub fn iomap(&mut self, addr: PhysAddr, pages: usize) -> Option<VirtAddr> {
        if self.pages < pages {
            return None;
        }

        let new_va = VirtAddr::new(self.start + self.offset * arch::PAGE_SIZE);

        kernel_page_table()
            .map(
                Some(MemRange::new(addr, arch::PAGE_SIZE * pages)),
                MemRange::new(new_va, arch::PAGE_SIZE * pages),
                MappingType::KernelDevice,
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

    *MMIO_ALLOCATOR.get() = new_allocator;

    println!(
        "Intialized mmio allocator {}",
        MMIO_ALLOCATOR.get().free_pages()
    );
}
