use crate::{
    arch::PT_LVL1_ENTIRES,
    arch::mm::mmu::{self, *},
    arch::{self, mm::mmu_flags},
    mm::allocators::page_alloc::page_allocator,
};
use rtl::arch::PAGE_SIZE;

use rtl::vmm::MappingType;
use rtl::vmm::types::*;

#[derive(Debug)]
pub enum MmError {
    Generic,
    InvalidAddr,
    NoMem,
    NotImpl,
    NoTranslation,
}

impl From<MmError> for () {
    fn from(_value: MmError) -> Self {}
}

pub struct PageFlags {
    flags: usize,
}

pub struct PageTableBlock {
    addr: VirtAddr,
    lvl: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct PageTableEntry(usize);

pub struct PageTable {
    base: VirtAddr,
}

impl PageTableBlock {
    pub fn new(addr: VirtAddr, lvl: u8) -> Self {
        Self { addr, lvl }
    }

    pub fn addr(&self) -> VirtAddr {
        self.addr
    }

    pub fn lvl(&self) -> u8 {
        self.lvl
    }

    pub fn is_last(&self) -> bool {
        self.lvl == arch::PAGE_TABLE_LVLS
    }

    pub fn is_valid_tte(&self, index: usize) -> bool {
        assert!(index < 512);

        PageTableEntry::from_bits(unsafe {
            self.addr.to_raw_mut::<usize>().add(index).read_volatile()
        })
        .valid()
    }

    pub unsafe fn set_tte(&mut self, index: usize, entry: PageTableEntry) {
        assert!(index < 512);

        unsafe {
            self.addr
                .to_raw_mut::<usize>()
                .add(index)
                .write_volatile(entry.bits());

            core::arch::asm!("dsb ishst", "isb");
        }
    }

    pub fn get_tte(&mut self, index: usize) -> PageTableEntry {
        assert!(index < 512);

        unsafe {
            PageTableEntry::from_bits(self.addr.to_raw_mut::<usize>().add(index).read_volatile())
        }
    }

    pub fn index_of(&self, addr: VirtAddr) -> usize {
        match self.lvl {
            1 => arch::mm::page_table::l1_linear_offset(addr),
            2 => arch::mm::page_table::l2_linear_offset(addr),
            3 => arch::mm::page_table::l3_linear_offset(addr),
            _ => panic!("Wrong page table block index"),
        }
    }

    pub fn next(&self, index: usize) -> Option<Self> {
        assert!(!self.is_last());

        let entry_next = unsafe {
            PageTableEntry::from_bits(self.addr.to_raw::<usize>().add(index).read_volatile())
        };

        if entry_next.valid() {
            Some(Self::new(VirtAddr::from(entry_next.addr()), self.lvl + 1))
        } else {
            None
        }
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
    pub const fn default() -> Self {
        Self {
            base: VirtAddr::new(0_usize),
        }
    }

    pub fn walk(&mut self, va: VirtAddr) {
        let mut base = self.lvl1();

        for _ in 0..2 {
            let index = base.index_of(va);
            println!("entry {:x}", base.get_tte(index).bits());

            let next_block = match base.next(index) {
                Some(e) => e,
                None => return,
            };

            base = next_block;
        }

        let index = base.index_of(va);
        println!("entry {:x}", base.get_tte(index).bits());
    }

    pub fn new() -> Option<Self> {
        let base: PhysAddr = page_allocator().alloc(1)?;
        let new_table = Self {
            base: VirtAddr::from(base),
        };

        let mut va = VirtAddr::from(base);
        unsafe { va.as_slice_mut::<u8>(PAGE_SIZE).fill(0x00) };

        Some(new_table)
    }

    fn set_leaf_tte(
        b: &mut PageTableBlock,
        index: usize,
        pa: PhysAddr,
        tp: MappingType,
        lvl: usize,
        _v: VirtAddr,
    ) {
        let flags = mmu::mapping_type_to_flags(tp);
        let control = if lvl != 3 {
            PageFlags::block().bits()
        } else {
            PageFlags::page().bits()
        };

        assert!(!b.is_valid_tte(index));

        unsafe {
            b.set_tte(
                index,
                PageTableEntry::from_bits(control | flags | pa.bits()),
            );
        };
    }

    fn allocate_new_block(
        b: &mut PageTableBlock,
        lvl: usize,
        index: usize,
    ) -> Result<PageTableBlock, MmError> {
        let new_page = page_allocator().alloc(1).ok_or(MmError::NoMem)?;
        let mut va = VirtAddr::from(new_page);

        unsafe { va.as_slice_mut::<u8>(PAGE_SIZE).fill(0x00) };

        let new_entry = PageTableEntry::from_bits(PageFlags::table().bits() | new_page.get());

        unsafe { b.set_tte(index, new_entry) };
        Ok(PageTableBlock::new(VirtAddr::from(new_page), lvl as u8 + 1))
    }

    fn abort_walk(
        _b: &mut PageTableBlock,
        _lvl: usize,
        _index: usize,
    ) -> Result<PageTableBlock, MmError> {
        Err(MmError::NoTranslation)
    }

    fn clean_tte(
        b: &mut PageTableBlock,
        index: usize,
        _pa: PhysAddr,
        _tp: MappingType,
        _lvl: usize,
        v: VirtAddr,
    ) {
        unsafe {
            b.set_tte(index, PageTableEntry::from_bits(0));
            flush_tlb_page_last(v);
        };
    }

    #[allow(clippy::too_many_arguments)]
    fn op_lvl<
        F: FnMut(&mut PageTableBlock, usize, PhysAddr, MappingType, usize, VirtAddr) + Copy, // Set leaf
        G: FnMut(&mut PageTableBlock, usize, usize) -> Result<PageTableBlock, MmError> + Copy, // Process walk
    >(
        mut base: PageTableBlock,
        lvl: usize,
        v: &mut MemRange<VirtAddr>,
        p: &mut MemRange<PhysAddr>,
        map: MappingType,
        mut cb: F,
        mut cb_b: G,
        use_huge_pages: bool,
    ) -> Result<VirtAddr, MmError> {
        let order = match lvl {
            1 => 30,
            2 => 21,
            3 => 12,
            _ => panic!("Kernel supports 3 lvl page table"),
        };
        let size = 1 << order;
        let res = v.start();

        assert!(v.size() == p.size());

        while {
            let index = base.index_of(v.start());

            if lvl < 3
                && !(use_huge_pages && v.start().is_aligned(order) && v.size().is_aligned(order))
            {
                let next_block = match base.next(index) {
                    Some(e) => e,
                    None => cb_b(&mut base, lvl, index)?,
                };

                Self::op_lvl(next_block, lvl + 1, v, p, map, cb, cb_b, use_huge_pages)?;
            } else {
                assert!(p.start().is_aligned(order));
                assert!(v.start().is_aligned(order));

                cb(&mut base, index, p.start(), map, lvl, v.start());

                p.truncate(size);
                v.truncate(size);
            }

            v.size() != 0 && index != 511
        } {}

        Ok(res)
    }

    pub fn map_hugepages(
        &mut self,
        p: Option<MemRange<PhysAddr>>,
        mut v: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<VirtAddr, MmError> {
        let mut p_range = if let Some(pr) = p {
            pr
        } else {
            MemRange::new(PhysAddr::from(v.start()), v.size())
        };

        Self::op_lvl(
            self.lvl1(),
            1,
            &mut v,
            &mut p_range,
            m_type,
            Self::set_leaf_tte,
            Self::allocate_new_block,
            true,
        )
    }

    pub fn map(
        &mut self,
        p: Option<MemRange<PhysAddr>>,
        mut v: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<VirtAddr, MmError> {
        let mut p_range = if let Some(pr) = p {
            pr
        } else {
            MemRange::new(PhysAddr::from(v.start()), v.size())
        };

        assert!(v.size() == p_range.size());

        Self::op_lvl(
            self.lvl1(),
            1,
            &mut v,
            &mut p_range,
            m_type,
            Self::set_leaf_tte,
            Self::allocate_new_block,
            false,
        )
    }

    pub fn free<F: Fn(PhysAddr, bool)>(
        &mut self,
        mut v: MemRange<VirtAddr>,
        cb: F,
    ) -> Result<(), MmError> {
        let mut p = MemRange::new(PhysAddr::new(v.start().bits()), v.size());

        Self::op_lvl(
            self.lvl1(),
            1,
            &mut v,
            &mut p,
            MappingType::NONE,
            |base, index, pa, tp, lvl, v| {
                let tte = base.get_tte(index);

                cb(
                    tte.addr(),
                    tte.flags().bits() == mmu::mapping_type_to_flags(MappingType::USER_DEVICE),
                );

                Self::clean_tte(base, index, pa, tp, lvl, v);
            },
            Self::abort_walk,
            true,
        )
        .map(|_| ())
    }

    #[inline]
    pub fn base(&self) -> PhysAddr {
        PhysAddr::from(self.base)
    }

    #[inline]
    fn entries_per_lvl(&self) -> usize {
        PT_LVL1_ENTIRES
    }

    #[inline]
    fn lvl1(&self) -> PageTableBlock {
        PageTableBlock::new(self.base, 1)
    }
}

impl PageTableEntry {
    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn from_bits(data: usize) -> Self {
        Self(data)
    }

    pub fn and(&mut self, data: usize) -> &mut Self {
        self.0 |= data;
        self
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & !mmu_flags::PAGE_ENTRY_FLAGS_MASK)
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
