use crate::{
    arch::mm::mmu,
    arch::PT_LVL1_ENTIRES,
    arch::{self, mm::mmu_flags},
    kernel::misc::*,
    mm::allocators::page_alloc::page_allocator,
    mm::paging::kernel_page_table::kernel_page_table,
};

use rtl::vmm::types::*;
use rtl::vmm::MappingType;
use rtl::arch::PAGE_SIZE;

#[derive(Debug)]
pub enum MmError {
    Generic,
    InvalidAddr,
    NoMem,
    NotImpl,
}

impl From<MmError> for () {
    fn from(_value: MmError) -> Self {
        ()
    }
}

impl From<()> for MmError {
    fn from(_value: ()) -> Self {
        MmError::Generic
    }
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

pub struct PageTable<const KERNEL: bool> {
    base: VirtAddr,
}

impl PageTableBlock {
    pub fn new(addr: VirtAddr, lvl: u8) -> Self {
        Self {
            addr: addr,
            lvl: lvl,
        }
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

    pub unsafe fn set_tte(&mut self, index: usize, entry: PageTableEntry) {
        assert!(index < 512);

        // println!("Set entry 0x{:x}", entry.bits());

        self.addr
            .to_raw_mut::<usize>()
            .offset(index as isize)
            .write_volatile(entry.bits());
        // TODO: barriers, please.....
    }

    pub fn tte(&self, index: usize) -> PageTableEntry {
        unsafe {
            PageTableEntry::from_bits(
                self.addr
                    .to_raw_mut::<usize>()
                    .offset(index as isize)
                    .read_volatile(),
            )
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
            PageTableEntry::from_bits(
                self.addr
                    .to_raw::<usize>()
                    .offset(index as isize)
                    .read_volatile(),
            )
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
        Self::from_bits(
            arch::mm::mmu_flags::BLOCK_VALID | arch::mm::mmu_flags::BLOCK_ACCESS_FLAG | 0b10,
        )
    }

    pub fn bits(&self) -> usize {
        self.flags
    }
}

impl<const KERNEL: bool> PageTable<KERNEL> {
    pub const fn default() -> Self {
        Self {
            base: VirtAddr::new(0_usize),
        }
    }

    pub fn new() -> Option<Self> {
        let base: PhysAddr = page_allocator().alloc(1)?.into();
        let mut new_table = Self {
            base: VirtAddr::from(base),
        };

        let base_va = new_table.base;

        // unsafe {
        //     core::slice::from_raw_parts_mut(base_va.to_raw_mut::<u8>(), PAGE_SIZE).fill(0);
        // }

        if KERNEL {
            new_table
                .map(
                    None,
                    MemRange::new(base_va, PAGE_SIZE),
                    MappingType::KERNEL_RWX,
                )
                .ok()?;
        } else {
            kernel_page_table()
                .map(
                    None,
                    MemRange::new(base_va, PAGE_SIZE),
                    MappingType::KERNEL_RWX,
                )
                .ok()?;
        }

        Some(new_table)
    }

    fn map_lvl(
        &mut self,
        mut base: PageTableBlock,
        lvl: usize,
        v: &mut MemRange<VirtAddr>,
        p: &mut MemRange<PhysAddr>,
        map: MappingType,
    ) -> Result<VirtAddr, MmError> {
        let size = match lvl {
            1 => _1GB,
            2 => _2MB,
            3 => _4KB,
            _ => unreachable!(),
        };
        let res = v.start();

        assert!(v.size() == p.size());

        while {
            let index = base.index_of(v.start());

            if lvl < 3 {
                let next_block = match base.next(index) {
                    Some(e) => e,
                    None => {
                        let new_page: PhysAddr = page_allocator()
                            .alloc(1)
                            .expect("Failed to allocate memory")
                            .into();
                        let new_entry =
                            PageTableEntry::from_bits(PageFlags::table().bits() | new_page.get());

                        unsafe { base.set_tte(index, new_entry) };

                        if !KERNEL {
                            kernel_page_table().map(
                                None,
                                MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                                MappingType::KERNEL_DATA,
                            )?;
                        }

                        self.map(
                            None,
                            MemRange::new(VirtAddr::from(new_page), PAGE_SIZE),
                            MappingType::KERNEL_DATA,
                        )?;

                        PageTableBlock::new(VirtAddr::from(new_page), lvl as u8 + 1)
                    }
                };

                self.map_lvl(next_block, lvl + 1, v, p, map)?;
            } else {
                let ao = p.start().bits();
                let flags = mmu::mapping_type_to_flags(map);

                unsafe {
                    base.set_tte(
                        index,
                        PageTableEntry::from_bits(PageFlags::block().bits() | flags | ao),
                    );
                };

                p.truncate(size);
                v.truncate(size);
            }

            v.size() != 0 && index != 511
        } {}

        Ok(res)
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

        self.map_lvl(self.lvl1(), 1, &mut v, &mut p_range, m_type)
    }

    pub fn unmap(&mut self, _v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
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
    pub fn valid_block() -> Self {
        Self(mmu_flags::BLOCK_ACCESS_FLAG | mmu_flags::BLOCK_VALID)
    }

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
