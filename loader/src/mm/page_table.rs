use super::alloc::alloc_pages;
use crate::arch::mmu::{lvl_to_order, va_to_index, Pte, PAGE_TABLE_LAST_LVL, PTE_COUNT};
use hal::address::{Address, MemRange, PhysAddr, VirtAddr};
use hal::arch::PAGE_SIZE;

pub struct PageTable {
    base: *mut Pte,
}

#[derive(Copy, Clone, Debug)]
pub enum PagePerms {
    Read,
    ReadWrite,
    Execute,
}

#[derive(Copy, Clone)]
pub enum PageKind {
    Normal,
    Device,
}

impl PageTable {
    pub fn new() -> Option<Self> {
        let base = alloc_pages(1)?.bits() as *mut Pte;

        Some(Self { base })
    }

    pub fn map_lvl(
        base: *mut Pte,
        va: &mut MemRange<VirtAddr>,
        pa: &mut MemRange<PhysAddr>,
        perms: PagePerms,
        kind: PageKind,
        lvl: usize,
    ) {
        let order = lvl_to_order(lvl);
        let size = 1 << order;

        while {
            let idx = va_to_index(va.start(), lvl);
            let pte = unsafe { base.add(idx).read() };

            if lvl != PAGE_TABLE_LAST_LVL {
                if pte.is_valid() {
                    let next = pte.pa().bits() as *mut _;
                    Self::map_lvl(next, va, pa, perms, kind, lvl + 1);
                } else {
                    let next = alloc_pages(1).expect("Failed to allocate memory for page table");
                    let next_pte = Pte::new_non_leaf(next);

                    unsafe { base.add(idx).write(next_pte) };

                    Self::map_lvl(next.bits() as *mut _, va, pa, perms, kind, lvl + 1);
                }
            } else {
                if pte.is_valid() {
                    panic!(
                        "Attempt to rewrite PTE at addr {:x} {:x}",
                        va.start(),
                        pte.bits()
                    );
                }

                unsafe { base.add(idx).write(Pte::make(pa.start(), perms, kind)) };
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
        kind: PageKind,
    ) {
        debug_assert_eq!(va.size(), pa.size());
        debug_assert_eq!(va.size().next_multiple_of(PAGE_SIZE), va.size());
        debug_assert_eq!(pa.size().next_multiple_of(PAGE_SIZE), pa.size());
        debug_assert!(va.start().is_page_aligned());
        debug_assert!(pa.start().is_page_aligned());

        Self::map_lvl(self.base, &mut va, &mut pa, perms, kind, 0)
    }

    pub fn base(&self) -> PhysAddr {
        PhysAddr::from_bits(self.base as usize)
    }
}
