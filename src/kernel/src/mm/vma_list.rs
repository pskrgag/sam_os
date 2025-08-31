use alloc::collections::BTreeSet;
use alloc::rc::Rc;
use core::cmp::Ordering;
use core::ops::{Deref, DerefMut};
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

#[derive(Debug, Eq, Clone, Copy)]
pub(crate) struct MemRangeVma(MemRange<VirtAddr>);

impl Deref for MemRangeVma {
    type Target = MemRange<VirtAddr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MemRangeVma {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum VmaFlags {
    Invalid,
    Allocated,
}

#[derive(Debug, Clone, Eq)]
pub struct Vma {
    pub(crate) range: MemRangeVma,
    pub(crate) tp: MappingType,
    state: VmaFlags,
}

pub struct VmaList {
    occupied: BTreeSet<Vma>,
    free: BTreeSet<MemRangeVma>,
}

impl MemRangeVma {
    pub fn invalid() -> Self {
        Self(MemRange::new(VirtAddr::from(usize::MAX), 0))
    }

    pub fn is_valid(&self) -> bool {
        self.start() != usize::MAX.into()
    }

    pub fn split_at(mut self, addr: VirtAddr, size: usize) -> [MemRangeVma; 3] {
        let range = self.0;
        let start = range.start();
        let isize = range.size();

        assert!(self.0.contains_addr(addr));
        assert!(self != Self::invalid());

        // Split at 3
        if addr != range.start() && addr.bits() != range.start() + range.size() - size {
            self = Self(MemRange::new(start, addr - start));

            let mut vma_middle = Self::new_fixed(addr, size);

            let vma_higer = MemRangeVma::new_fixed(
                VirtAddr::new(addr + size),
                isize - self.0.size() - vma_middle.0.size(),
            );

            [self, vma_middle, vma_higer]
        } else if addr == range.start() {
            let vma_lower = MemRangeVma::new_fixed(addr, size);

            self = MemRangeVma::new_fixed(VirtAddr::new(addr + size), range.size() - size);

            [vma_lower, self, Self::invalid()]
        } else {
            self = MemRangeVma::new_fixed(range.start(), range.size() - size);

            let vma_higer = MemRangeVma::new_fixed(addr, size);
            [self, vma_higer, Self::invalid()]
        }
    }
}

impl VmaList {
    #[cfg(test)]
    pub fn vma_list_sorted(&self) -> alloc::vec::Vec<Vma> {
        self.occupied.iter().map(|x| x.clone()).collect()
    }

    pub fn new() -> Self {
        Self {
            free: BTreeSet::from([MemRangeVma::max_user()]),
            occupied: BTreeSet::new(),
        }
    }

    pub fn add_to_tree(&mut self, vma: Vma) -> Result<VirtAddr, ()> {
        let mut vma = vma;

        let range = if !vma.is_fixed() {
            let range = self.free_range(vma.size(), None).ok_or(())?;

            // Update range with new address
            vma.range = MemRangeVma(MemRange {
                start: range.start(),
                size: vma.size(),
            });

            range
        } else {
            self.free_range(vma.size(), Some(vma.range.0.start))
                .ok_or(())?
        };

        let splitted = range.split_at(vma.start(), vma.size());

        for i in splitted {
            if i.start() == vma.start() {
                assert!(self.occupied.insert(vma.clone()));
            } else if i.is_valid() {
                assert!(!i.start().is_null());
                // TODO: Handle contiguous regions to fix fragmentation
                self.free.insert(i);
            }
        }

        Ok(vma.start())
    }

    pub fn free(&mut self, vma: Vma) -> Option<()> {
        todo!()
    }

    fn free_range(&mut self, size: usize, addr: Option<VirtAddr>) -> Option<MemRangeVma> {
        if let Some(addr) = addr {
            Some(
                self.free
                    .take(&MemRangeVma(MemRange::new(addr, 1)))?
                    .clone(),
            )
        } else {
            for i in &self.free {
                if i.0.size >= size {
                    return Some(self.free.take(&MemRangeVma(i.0)).unwrap().clone());
                }
            }

            None
        }
    }
}

impl Vma {
    pub fn new(range: MemRangeVma, tp: MappingType) -> Self {
        assert!(range.0.start().is_page_aligned());

        Self {
            range,
            tp,
            state: VmaFlags::Allocated,
        }
    }

    fn from_range(r: MemRange<VirtAddr>) -> Self {
        Self {
            range: MemRangeVma(r),
            tp: MappingType::NONE,
            state: VmaFlags::Invalid,
        }
    }

    pub fn default_user() -> Self {
        Self::new(MemRangeVma::max_user(), MappingType::NONE)
    }

    pub fn invalid() -> Self {
        Self {
            range: MemRangeVma::new_fixed(0.into(), 0),
            tp: MappingType::USER_DEVICE,
            state: VmaFlags::Invalid,
        }
    }

    pub fn merge(&mut self, other: Vma) -> Option<()> {
        let self_before = other.start() == (self.start() + self.size()).into();
        let self_after = self.start() == (other.start() + other.size()).into();
        let flags_eq = self.state == other.state && self.tp == other.tp;

        assert!(self_before != self_after);

        if !(self_before || self_after) || !flags_eq {
            None
        } else if self_before {
            self.range = MemRangeVma::new_fixed(self.start(), self.size() + other.size());
            Some(())
        } else {
            self.range = MemRangeVma::new_fixed(other.start(), self.size() + other.size());
            Some(())
        }
    }

    pub fn mark_allocated(&mut self) {
        self.state = VmaFlags::Allocated;
    }

    pub fn is_valid(&self) -> bool {
        self.state != VmaFlags::Invalid
    }

    pub fn contains_addr(&self, addr: VirtAddr) -> bool {
        self.range.0.contains_addr(addr)
    }

    pub fn contains_range(&self, addr: MemRange<VirtAddr>) -> bool {
        self.range.0.contains_range(addr)
    }

    pub fn start(&self) -> VirtAddr {
        self.range.0.start()
    }

    pub fn size(&self) -> usize {
        self.range.0.size()
    }

    pub fn map_flags(&self) -> MappingType {
        self.tp
    }

    pub fn is_fixed(&self) -> bool {
        !self.range.0.start().is_null()
    }
}

impl MemRangeVma {
    pub fn new_fixed(addr: VirtAddr, size: usize) -> Self {
        Self(MemRange::new(addr, size))
    }

    pub fn new_non_fixed(size: usize) -> Self {
        Self(MemRange::new(VirtAddr::new(0), size))
    }

    pub fn max_user() -> Self {
        Self(MemRange::max_user())
    }
}

impl From<MemRange<VirtAddr>> for MemRangeVma {
    fn from(value: MemRange<VirtAddr>) -> Self {
        Self(value)
    }
}

impl PartialEq for MemRangeVma {
    fn eq(&self, other: &Self) -> bool {
        self.0.contains_addr(other.0.start())
    }
}

impl PartialOrd for MemRangeVma {
    fn partial_cmp(&self, other: &MemRangeVma) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemRangeVma {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0.contains_addr(other.0.start()) || other.0.contains_addr(self.0.start()) {
            Ordering::Equal
        } else {
            self.0.cmp(&other.0)
        }
    }
}

impl PartialEq for Vma {
    fn eq(&self, other: &Self) -> bool {
        self.range.eq(&other.range)
    }
}

impl PartialOrd for Vma {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.range.partial_cmp(&other.range)
    }
}

impl Ord for Vma {
    fn cmp(&self, other: &Self) -> Ordering {
        self.range.cmp(&other.range)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use rtl::arch::PAGE_SIZE;
    use rtl::vmm::types::*;
    use test_macros::*;

    #[kernel_test]
    fn vma_list_empty() {
        let mut list = VmaList::new();

        test_assert!(list
            .free_range(MemRange::<VirtAddr>::max_user().size())
            .is_some());
    }

    #[kernel_test]
    fn vma_list_add() {
        let mut list = VmaList::new();
        let fixed_va = VirtAddr::new(0x2000);

        let va = list.add_to_tree(Vma::new(
            MemRangeVma::new_fixed(fixed_va, 0x1000),
            MappingType::USER_DATA,
        ));

        test_assert!(va.is_ok());
        test_assert_eq!(va.unwrap(), fixed_va);
        test_assert_eq!(list.vma_list_sorted().len(), 3);
    }

    #[kernel_test]
    fn vma_list_add_nofixed() {
        let mut list = VmaList::new();

        test_assert!(list
            .add_to_tree(Vma::new(
                MemRangeVma::new_fixed(VirtAddr::new(0), 0x1000),
                MappingType::USER_DATA,
            ))
            .is_ok());
        test_assert_eq!(list.vma_list_sorted().len(), 2);
    }

    #[kernel_test]
    fn vma_list_free() {
        let mut list = VmaList::new();
        let vma = Vma::new(
            MemRangeVma::new_fixed(0x2000.into(), 0x1000),
            MappingType::USER_DATA,
        );

        let va = list.add_to_tree(vma.clone());

        test_assert!(va.is_ok());
        test_assert!(list.add_to_tree(vma.clone()).is_err());

        list.free(vma.clone());
        test_assert_eq!(list.vma_list_sorted().len(), 1);

        test_assert!(list.add_to_tree(vma).is_ok());
    }
}
