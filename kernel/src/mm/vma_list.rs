use crate::mm::{paging::page_table::MappingType, types::*};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum VmaFlags {
    VmaFixed,
    VmaFree,
    VmaUncommited,
}

#[derive(Copy, Clone)]
pub struct Vma {
    pub(crate) range: MemRange<VirtAddr>,
    pub(crate) tp: MappingType,
    pub(crate) flags: VmaFlags,
}

pub struct VmaList {
    list: BTreeSet<Vma>,
}

#[inline]
fn do_intersect(range1: &MemRange<VirtAddr>, range2: &MemRange<VirtAddr>) -> bool {
    (range1.start() + range1.size()) < range2.start().into()
        && (range2.start() + range2.size()) < range1.start().into()
}

impl VmaList {
    pub fn new() -> Self {
        let mut list = BTreeSet::new();

        list.insert(Vma::default_user());
        Self { list }
    }

    fn find_free_vma_addr(&self, addr: VirtAddr) -> Vma {
        *self.list.iter().find(|&x| x.contains_addr(addr)).unwrap()
    }

    pub fn add_to_tree(&mut self, vma: Vma) -> Option<VirtAddr> {
        let vma = self.find_free_vma_addr(vma.start());
        let addr = if vma.start() == VirtAddr::new(0) {
            self.free_range(vma.size())?.start()
        } else {
            vma.start()
        };

        if vma.is_free() {
            self.list.remove(&vma);

            let vmas = vma.split_at(addr, vma.size(), vma.map_flags());

            for i in vmas {
                self.list.insert(i);
            }
        }

        Some(addr)
    }

    pub fn free_range(&self, size: usize) -> Option<MemRange<VirtAddr>> {
        for i in &self.list {
            if i.is_free() && i.size() >= size {
                return Some(MemRange::new(i.start(), size));
            }
        }

        None
    }
}

impl Vma {
    pub fn new(range: MemRange<VirtAddr>, tp: MappingType) -> Self {
        assert!(range.start().is_page_aligned());

        Self {
            range,
            tp,
            flags: VmaFlags::VmaFree,
        }
    }

    pub fn split_at(self, addr: VirtAddr, size: usize, tp: MappingType) -> Vec<Vma> {
        let range = &self.range;
        let start = range.start();
        let end = start + range.size();

        if addr != range.start() && addr.bits() != range.start() + range.size() - size {
            let mut v = Vec::with_capacity(3);

            v.push(Vma::new(MemRange::new(start, addr - start), self.tp));
            v.push(Vma::new(MemRange::new(addr, size), tp));
            v.push(Vma::new(
                MemRange::new(VirtAddr::new(addr + size), end - addr.bits() + size),
                self.tp,
            ));

            v
        } else if addr == range.start() {
            let mut v = Vec::with_capacity(2);

            v.push(Vma::new(MemRange::new(addr, size), tp));
            v.push(Vma::new(
                MemRange::new(VirtAddr::new(addr + size), range.size() - size),
                self.tp,
            ));
            v
        } else {
            let mut v = Vec::with_capacity(2);

            v.push(Vma::new(
                MemRange::new(range.start(), range.size() - size),
                self.tp,
            ));
            v.push(Vma::new(MemRange::new(addr, size), tp));

            v
        }
    }

    pub fn is_free(&self) -> bool {
        self.flags == VmaFlags::VmaFree
    }

    pub fn contains_addr(&self, addr: VirtAddr) -> bool {
        self.range.contains(addr)
    }

    pub fn start(&self) -> VirtAddr {
        self.range.start()
    }

    pub fn size(&self) -> usize {
        self.range.size()
    }

    pub fn map_flags(&self) -> MappingType {
        self.tp
    }

    pub fn default_user() -> Vma {
        Self::new(MemRange::max_user(), MappingType::None)
    }
}

impl PartialEq for Vma {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range
    }
}

impl PartialOrd for Vma {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.range.cmp(&other.range))
    }
}

impl Eq for Vma {}

impl Ord for Vma {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.range.cmp(&other.range)
    }
}
