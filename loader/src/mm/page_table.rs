use super::alloc::alloc_pages;
use crate::arch::mmu::{lvl_to_size, va_to_index, Pte, PAGE_TABLE_LEVELS, PTE_COUNT};
use rtl::vmm::types::{Address, MemRange, PhysAddr, VirtAddr};

pub struct PageTable {
    base: *mut Pte,
}

#[derive(Copy, Clone)]
pub enum PagePerms {
    Read,
    ReadWrite,
    Execute,
}

impl PageTable {
    pub fn new() -> Option<Self> {
        let base = alloc_pages(1)?.bits() as *mut Pte;

        // Establish self-mapping
        unsafe {
            base.add(PTE_COUNT - 1)
                .write(Pte::new_non_leaf(PhysAddr::new(base as usize)))
        };

        Some(Self { base })
    }

    pub fn map_lvl(
        base: *mut Pte,
        va: &mut MemRange<VirtAddr>,
        pa: &mut MemRange<PhysAddr>,
        perms: PagePerms,
        lvl: usize,
    ) {
        let order = lvl_to_size(lvl);
        let size = 1 << order;

        while {
            let idx = va_to_index(va.start(), lvl);
            let pte = unsafe { base.add(idx).read() };

            if lvl != PAGE_TABLE_LEVELS {
                if pte.is_valid() {
                    let next = pte.pa().bits() as *mut _;
                    Self::map_lvl(next, va, pa, perms, lvl + 1);
                } else {
                    let next = alloc_pages(1).expect("Failed to allocate memory for page table");
                    let next_pte = Pte::new_non_leaf(next);

                    unsafe { base.add(idx).write(next_pte) };

                    Self::map_lvl(next.bits() as *mut _, va, pa, perms, lvl + 1);
                }
            } else {
                assert!(!pte.is_valid());

                unsafe { base.add(idx).write(Pte::make(pa.start(), perms)) };
                pa.truncate(size);
                va.truncate(size);
            }

            va.size() != 0 && idx != (PTE_COUNT - 1)
        } {}
    }

    pub fn map_pages(
        &mut self,
        mut va: MemRange<VirtAddr>,
        mut pa: MemRange<PhysAddr>,
        perms: PagePerms,
    ) {
        assert!(va.size() == pa.size());
        assert!(va.start().is_page_aligned());
        assert!(va.size().is_page_aligned());
        assert!(pa.size().is_page_aligned());
        assert!(pa.start().is_page_aligned());

        Self::map_lvl(self.base, &mut va, &mut pa, perms, 1)
    }

    pub fn base(&self) -> PhysAddr {
        PhysAddr::new(self.base as usize)
    }
}
