use crate::{
    arch::mm::mmu::{self, *},
    arch::{self, mm::mmu_flags},
    mm::pmm::page_alloc::page_allocator,
    mm::pmm::page_list::PageListIterator,
};
use hal::address::*;
use hal::arch::PAGE_SIZE;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub struct PageFlags {
    flags: usize,
}

pub struct PageTableBlock {
    addr: LinearAddr,
    lvl: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct PageTableEntry(usize);

pub struct PageTable {
    base: LinearAddr,
    is_user: bool,
}

fn empty_page_source() -> Option<&'static mut impl PageSource> {
    None::<&mut MemRange<PhysAddr>>
}

impl PageTableBlock {
    pub fn new(addr: LinearAddr, lvl: u8) -> Self {
        Self { addr, lvl }
    }

    pub fn addr(&self) -> LinearAddr {
        self.addr
    }

    pub fn lvl(&self) -> u8 {
        self.lvl
    }

    pub fn is_last(&self) -> bool {
        self.lvl == arch::PAGE_TABLE_LVLS
    }

    pub fn is_valid_pte(&self, index: usize) -> bool {
        assert!(index < 512);

        let va: VirtAddr = self.addr.into();
        PageTableEntry::from_bits(unsafe { va.to_raw_mut::<usize>().add(index).read_volatile() })
            .valid()
    }

    pub unsafe fn set_pte(&mut self, index: usize, entry: PageTableEntry) {
        assert!(index < 512);

        let va: VirtAddr = self.addr.into();
        unsafe {
            va.to_raw_mut::<usize>()
                .add(index)
                .write_volatile(entry.bits());

            core::arch::asm!("dsb ishst", "isb");
        }
    }

    pub fn get_pte(&mut self, index: usize) -> PageTableEntry {
        assert!(index < 512);

        let va: VirtAddr = self.addr.into();
        unsafe { PageTableEntry::from_bits(va.to_raw_mut::<usize>().add(index).read_volatile()) }
    }

    pub fn pte_addr(&self, index: usize) -> *const PageTableEntry {
        let va: VirtAddr = self.addr.into();
        unsafe { va.to_raw_mut::<usize>().add(index) as _ }
    }

    pub fn index_of(&self, addr: VirtAddr) -> usize {
        match self.lvl {
            0 => arch::mm::page_table::l0_linear_offset(addr),
            1 => arch::mm::page_table::l1_linear_offset(addr),
            2 => arch::mm::page_table::l2_linear_offset(addr),
            3 => arch::mm::page_table::l3_linear_offset(addr),
            _ => panic!("Wrong page table block index"),
        }
    }

    pub fn next(&self, index: usize) -> Option<Self> {
        if self.is_last() {
            None
        } else {
            let va: VirtAddr = self.addr.into();
            let entry_next = unsafe {
                PageTableEntry::from_bits(va.to_raw::<usize>().add(index).read_volatile())
            };

            if entry_next.valid() {
                Some(Self::new(LinearAddr::from(entry_next.addr()), self.lvl + 1))
            } else {
                None
            }
        }
    }
}

pub trait PageSource {
    fn next_page(&mut self) -> Option<PhysAddr>;
}

impl PageSource for MemRange<PhysAddr> {
    fn next_page(&mut self) -> Option<PhysAddr> {
        if self.size != 0 {
            let page = self.start;

            self.size -= PAGE_SIZE;
            self.start = self.start + PAGE_SIZE;
            Some(page)
        } else {
            None
        }
    }
}

impl PageSource for PageListIterator<'_> {
    fn next_page(&mut self) -> Option<PhysAddr> {
        self.next().map(|x| x.into())
    }
}

impl PageFlags {
    pub fn from_bits(bits: usize) -> Self {
        Self { flags: bits }
    }

    pub fn table() -> Self {
        Self::from_bits(arch::mm::mmu_flags::TABLE_VALID)
    }

    pub fn block() -> Self {
        Self::from_bits(arch::mm::mmu_flags::BLOCK_VALID | arch::mm::mmu_flags::BLOCK_ACCESS_FLAG)
    }

    pub fn page() -> Self {
        Self::from_bits(arch::mm::mmu_flags::PAGE_VALID | arch::mm::mmu_flags::BLOCK_ACCESS_FLAG)
    }

    pub fn bits(&self) -> usize {
        self.flags
    }
}

impl PageTable {
    pub unsafe fn from(base: PhysAddr) -> Self {
        Self {
            base: LinearAddr::from(base),
            is_user: false,
        }
    }

    pub fn translate(&self, va: VirtAddr) -> Option<PhysAddr> {
        let mut base = self.lvl0();

        for lvl in 0..=arch::PAGE_TABLE_LVLS {
            let index = base.index_of(va);

            if lvl != arch::PAGE_TABLE_LVLS {
                let next_block = match base.next(index) {
                    Some(e) => e,
                    None => return None,
                };

                base = next_block;
            } else {
                return Some(base.get_pte(index).addr());
            }
        }

        panic!("")
    }

    pub fn new() -> Option<Self> {
        let base: PhysAddr = page_allocator()
            .alloc_pages(1)?
            .pop_front()
            .unwrap()
            .pfn()
            .into();
        let new_table = Self {
            base: LinearAddr::from(base),
            is_user: true,
        };

        Some(new_table)
    }

    fn set_leaf_pte(
        b: &mut PageTableBlock,
        index: usize,
        pa: PhysAddr,
        tp: MappingType,
        lvl: u8,
        v: VirtAddr,
        is_user: bool,
    ) {
        let flags = mmu::mapping_type_to_flags(tp, is_user);
        let control = if lvl != 3 {
            PageFlags::block().bits()
        } else {
            PageFlags::page().bits()
        };

        assert!(
            !b.is_valid_pte(index),
            "PTE addr {:p}, PTE content {:x}",
            b.pte_addr(index),
            b.get_pte(index).bits()
        );

        unsafe {
            b.set_pte(
                index,
                PageTableEntry::from_bits(control | flags | pa.bits()),
            );

            // TODO: this is bad... Need to check if PTE was valid. Otherwise flush is redundant
            flush_tlb_page_last(v);
        };
    }

    fn allocate_new_block(
        b: &mut PageTableBlock,
        lvl: u8,
        index: usize,
    ) -> Result<PageTableBlock, ErrorType> {
        let new_page: PhysAddr = page_allocator()
            .alloc_pages(1)
            .ok_or(ErrorType::NoMemory)?
            .pop_front()
            .unwrap()
            .pfn()
            .into();
        let new_entry = PageTableEntry::from_bits(PageFlags::table().bits() | new_page.bits());

        unsafe { b.set_pte(index, new_entry) };
        Ok(PageTableBlock::new(LinearAddr::from(new_page), lvl + 1))
    }

    fn abort_walk(
        _b: &mut PageTableBlock,
        _lvl: u8,
        _index: usize,
    ) -> Result<PageTableBlock, ErrorType> {
        Err(ErrorType::Generic)
    }

    fn clean_tte(
        b: &mut PageTableBlock,
        index: usize,
        _pa: PhysAddr,
        _tp: MappingType,
        _lvl: u8,
        v: VirtAddr,
    ) {
        unsafe {
            b.set_pte(index, PageTableEntry::from_bits(0));
            flush_tlb_page_last(v);
        };
    }

    fn protect_leaf_pte(
        b: &mut PageTableBlock,
        index: usize,
        pa: PhysAddr,
        tp: MappingType,
        lvl: u8,
        v: VirtAddr,
        is_user: bool,
    ) {
        let flags = mmu::mapping_type_to_flags(tp, is_user);
        let control = if lvl != 3 {
            PageFlags::block().bits()
        } else {
            PageFlags::page().bits()
        };

        assert!(b.is_valid_pte(index));

        unsafe {
            b.set_pte(
                index,
                PageTableEntry::from_bits(control | flags | pa.bits()),
            );
            flush_tlb_page_last(v);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn op_lvl<
        F: FnMut(&mut PageTableBlock, usize, PhysAddr, MappingType, u8, VirtAddr, bool) + Copy, // Set leaf
        G: FnMut(&mut PageTableBlock, u8, usize) -> Result<PageTableBlock, ErrorType> + Copy, // Process walk
    >(
        mut base: PageTableBlock,
        lvl: u8,
        v: &mut MemRange<VirtAddr>,
        mut p: Option<&mut impl PageSource>,
        map: MappingType,
        mut cb: F,
        mut cb_b: G,
        use_huge_pages: bool,
        is_user: bool,
    ) -> Result<VirtAddr, ErrorType> {
        let order = match lvl {
            0 => 39,
            1 => 30,
            2 => 21,
            3 => 12,
            _ => panic!("Kernel supports 4 lvl page table"),
        };
        let size = 1 << order;
        let res = v.start();

        while {
            let index = base.index_of(v.start());

            if lvl < arch::PAGE_TABLE_LVLS
                && !(use_huge_pages
                    && v.start().is_aligned(order)
                    && v.size().next_multiple_of(1 << order) == v.size())
            {
                let next_block = match base.next(index) {
                    Some(e) => e,
                    None => cb_b(&mut base, lvl, index)?,
                };

                Self::op_lvl(
                    next_block,
                    lvl + 1,
                    v,
                    p.as_deref_mut(),
                    map,
                    cb,
                    cb_b,
                    use_huge_pages,
                    is_user,
                )?;
            } else {
                debug_assert!(v.start().is_aligned(order));

                let pa = match p.as_deref_mut() {
                    Some(source) => source.next_page().ok_or(ErrorType::InvalidArgument)?,
                    None => base.get_pte(index).addr(),
                };

                cb(
                    &mut base,
                    index,
                    pa,
                    map,
                    lvl,
                    v.start(),
                    is_user,
                );

                v.truncate(size);
            }

            v.size() != 0 && index != (arch::PTE_PER_PAGE - 1)
        } {}

        Ok(res)
    }

    fn map_internal(
        &mut self,
        mut p: impl PageSource,
        mut v: MemRange<VirtAddr>,
        m_type: MappingType,
        hp: bool,
    ) -> Result<VirtAddr, ErrorType> {
        Self::op_lvl(
            self.lvl0(),
            0,
            &mut v,
            Some(&mut p),
            m_type,
            Self::set_leaf_pte,
            Self::allocate_new_block,
            hp,
            self.is_user,
        )
    }

    pub fn map(
        &mut self,
        p: impl PageSource,
        v: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<VirtAddr, ErrorType> {
        self.map_internal(p, v, m_type, false)
    }

    pub fn protect(
        &mut self,
        mut v: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<(), ErrorType> {
        Self::op_lvl(
            self.lvl0(),
            0,
            &mut v,
            empty_page_source(),
            m_type,
            Self::protect_leaf_pte,
            Self::abort_walk,
            true,
            self.is_user,
        )
        .map(|_| ())
    }

    pub fn map_linear(
        &mut self,
        p: MemRange<PhysAddr>,
        m_type: MappingType,
    ) -> Result<VirtAddr, ErrorType> {
        let v_range: MemRange<LinearAddr> = MemRange::new(p.start().into(), p.size());
        let va_range: MemRange<VirtAddr> = MemRange::new(v_range.start().into(), v_range.size());

        self.map_internal(p, va_range, m_type, false)
    }

    pub fn free<F: Fn(PhysAddr)>(
        &mut self,
        mut v: MemRange<VirtAddr>,
        cb: F,
    ) -> Result<(), ErrorType> {
        Self::op_lvl(
            self.lvl0(),
            0,
            &mut v,
            empty_page_source(),
            MappingType::None,
            |base, index, pa, tp, lvl, v, _| {
                let tte = base.get_pte(index);

                cb(tte.addr());
                Self::clean_tte(base, index, pa, tp, lvl, v);
            },
            Self::abort_walk,
            true,
            self.is_user,
        )
        .map(|_| ())
    }

    #[inline]
    pub fn base(&self) -> PhysAddr {
        PhysAddr::from(self.base)
    }

    #[inline]
    fn lvl0(&self) -> PageTableBlock {
        PageTableBlock::new(self.base, 0)
    }
}

impl PageTableEntry {
    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn from_bits(data: usize) -> Self {
        Self(data)
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::from_bits(self.0 & !mmu_flags::PAGE_ENTRY_FLAGS_MASK)
    }

    pub fn flags(&self) -> PageFlags {
        PageFlags::from_bits(self.0 & mmu_flags::PAGE_ENTRY_FLAGS_MASK)
    }

    pub fn valid(&self) -> bool {
        self.0 & 0b11 != 0
    }
}

impl core::fmt::Debug for PageTableBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "PageTableBlock [ base: 0x{:x} ]",
            self.addr.bits()
        ))
    }
}
