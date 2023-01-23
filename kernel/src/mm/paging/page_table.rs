use crate::{
    arch::PT_LVL1_ENTIRES,
    arch::{self, mm::mmu_flags},
    arch::{mm::mmu, PAGE_SIZE},
    kernel::locking::spinlock::Spinlock,
    kernel::misc::*,
    mm::{allocators::page_alloc::page_allocator, types::*},
};

use alloc::boxed::Box;
use core::pin::Pin;

#[derive(Debug)]
pub enum MmError {
    InvalidAddr,
    NoMem,
    NotImpl,
}

#[derive(Clone, Copy)]
pub enum MappingType {
    KernelData,
    KernelText,
    KernelDataRo,
    KernelRWX,
    KernelDevice,
    KernelNothing,
    UserData,
    UserText,
    UserDataRo,
}

pub struct PageFlags {
    flags: usize,
}

pub struct PageTableBlock<const LVL: u8, const N: usize> {
    block: [PageTableEntry; N],
}

pub type PDir<const LVL: u8, const N: usize> = Pin<Box<PageTableBlock<LVL, N>, PageBlockAllocator>>;

#[derive(Clone, Copy, Debug)]
pub struct PageTableEntry(usize);

pub struct PageTable<const N: usize = 512> {
    dir: PDir<1, N>,
    kernel: bool,
}

pub struct PageBlockAllocator;

unsafe impl core::alloc::Allocator for PageBlockAllocator {
    fn allocate(
        &self,
        _layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        let va = VirtAddr::from(PhysAddr::from(page_allocator().alloc(1).unwrap()));

        Ok(unsafe {
            core::ptr::NonNull::new(core::slice::from_raw_parts_mut(va.to_raw_mut::<u8>(), 4096))
                .unwrap()
        })
    }

    unsafe fn deallocate(&self, _ptr: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        // Do nothing;
        // We don't free kernel page table blocks
    }
}

impl<const LVL: u8, const N: usize> PageTableBlock<LVL, N> {
    pub fn new() -> Pin<Box<Self, PageBlockAllocator>> {
        Box::<Self, PageBlockAllocator>::pin_in(
            Self {
                block: [PageTableEntry(0); N],
            },
            PageBlockAllocator {},
        )
    }

    pub unsafe fn from_raw(addr: VirtAddr) -> PDir<LVL, N> {
        assert!(addr.is_page_aligned());

        Box::<Self, PageBlockAllocator>::from_raw_in(
            addr.to_raw_mut::<Self>(),
            PageBlockAllocator {},
        )
        .into()
    }

    pub fn addr(&self) -> VirtAddr {
        VirtAddr::from_raw(&self.block as *const _)
    }

    pub fn lvl(&self) -> usize {
        LVL as usize
    }

    pub fn is_last(&self) -> bool {
        LVL == arch::PAGE_TABLE_LVLS
    }

    pub unsafe fn set_tte(&mut self, index: usize, entry: PageTableEntry) {
        assert!(index < 512);

        (&mut self.block[index].0 as *mut usize).write_volatile(entry.bits());

        unsafe {
            crate::arch::barriers::wb();
            crate::arch::barriers::isb();
        }
    }

    pub fn tte(&self, index: usize) -> PageTableEntry {
        unsafe { PageTableEntry((&self.block[index].0 as *const usize).read_volatile()) }
    }

    pub fn index_of(&self, addr: VirtAddr) -> usize {
        match LVL {
            1 => arch::mm::page_table::l1_linear_offset(addr),
            2 => arch::mm::page_table::l2_linear_offset(addr),
            3 => arch::mm::page_table::l3_linear_offset(addr),
            _ => panic!("Wrong page table block index"),
        }
    }

    pub fn next(&self, index: usize) -> Option<PDir<{ LVL + 1 }, N>> {
        assert!(!self.is_last());

        let entry_next = unsafe {
            PageTableEntry::from_bits((&self.block[index].0 as *const usize).read_volatile())
        };

        if entry_next.valid() {
            unsafe {
                Some(PageTableBlock::<{ LVL + 1 }, N>::from_raw(VirtAddr::from(
                    entry_next.addr(),
                )))
            }
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

impl<const N: usize> PageTable<N> {
    pub fn new() -> Self {
        Self {
            dir: PageTableBlock::new(),
            kernel: false,
        }
    }

    /* If p is None then caller want linear mapping */
    pub fn map(
        &mut self,
        p: Option<MemRange<PhysAddr>>,
        v: MemRange<VirtAddr>,
        m_type: MappingType,
    ) -> Result<(), MmError> {
        let flags = mmu::mapping_type_to_flags(m_type);
        let mut lvl1_sz = v.size();
        let mut va = v.start();
        let pa = if let Some(range) = p {
            assert!(range.size() == v.size());
            assert!(range.start().is_page_aligned());
            Some(range.start())
        } else {
            None
        };

        assert!(v.start().is_page_aligned());

        if v.size() == 0 {
            return Ok(());
        }

        /* Lvl1 loop */
        while {
            let table_block_1 = self.lvl1();
            let lvl1_index = table_block_1.index_of(va);
            let mut table_block_2 = match table_block_1.next(lvl1_index) {
                Some(e) => e,
                None => {
                    let new_block = PageTableBlock::<2, N>::new();
                    let new_entry = PageTableEntry::from_bits(
                        PageFlags::table().bits() | PhysAddr::from(new_block.addr()).bits(),
                    );
                    unsafe { table_block_1.set_tte(lvl1_index, new_entry) };

                    new_block
                }
            };
            let mut lvl2_sz = if lvl1_sz > _1GB { _1GB } else { lvl1_sz };
            assert!(table_block_1.lvl() == 1);

            while {
                let mut lvl3_sz = if lvl2_sz > _2MB { _2MB } else { lvl2_sz };
                let lvl2_index = table_block_2.index_of(va);
                let mut table_block_3 = match table_block_2.next(lvl2_index) {
                    Some(e) => e,
                    None => {
                        let new_block = PageTableBlock::<3, N>::new();
                        let new_entry = PageTableEntry::from_bits(
                            PageFlags::table().bits() | PhysAddr::from(new_block.addr()).bits(),
                        );

                        unsafe { table_block_2.set_tte(lvl2_index, new_entry) };

                        new_block
                    }
                };

                assert!(table_block_2.lvl() == 2);

                while {
                    let lvl3_index: usize = table_block_3.index_of(va);
                    let ao = if let Some(addr) = pa {
                        addr.get()
                    } else {
                        PhysAddr::from(va).get()
                    };

                    assert!(!table_block_3.tte(lvl3_index).valid());

                    unsafe {
                        table_block_3.set_tte(
                            lvl3_index,
                            PageTableEntry::from_bits(PageFlags::block().bits() | flags | ao),
                        );
                    };

                    assert!(table_block_3.lvl() == 3);

                    va.add(arch::PAGE_SIZE);

                    if pa.is_some() {
                        pa.unwrap().add(arch::PAGE_SIZE);
                    }

                    lvl3_sz -= _4KB;
                    lvl3_sz != 0
                } {}

                if lvl2_sz < _2MB {
                    return Ok(());
                }

                lvl2_sz -= _2MB;
                lvl2_sz != 0
            } {}

            if lvl1_sz < _1GB {
                return Ok(());
            }
            lvl1_sz -= _1GB;
            lvl1_sz != 0
        } {}

        unreachable!();
    }

    pub fn unmap(&mut self, _v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    #[inline]
    pub fn base(&self) -> PhysAddr {
        PhysAddr::from(self.dir.addr())
    }

    #[inline]
    fn entries_per_lvl(&self) -> usize {
        PT_LVL1_ENTIRES
    }

    #[inline]
    fn lvl1(&mut self) -> &mut PDir<1, N> {
        &mut self.dir
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
