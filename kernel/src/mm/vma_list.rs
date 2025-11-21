use alloc::collections::BTreeSet;
use core::cmp::Ordering;
use core::ops::{Deref, DerefMut};
use rtl::error::ErrorType;
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

#[derive(Debug)]
struct FreeRegions(BTreeSet<MemRangeVma>);

impl FreeRegions {
    fn find_by_size(&mut self, size: usize) -> Option<MemRangeVma> {
        for i in &self.0 {
            if i.0.size >= size {
                return Some(self.0.take(&MemRangeVma(i.0)).unwrap().clone());
            }
        }

        None
    }

    fn find_by_addr(&mut self, addr: VirtAddr) -> Option<MemRangeVma> {
        Some(self.0.take(&MemRangeVma(MemRange::new(addr, 1)))?.clone())
    }

    fn insert(&mut self, range: MemRangeVma) {
        assert!(range.start().is_page_aligned());
        assert!(range.size().is_page_aligned());

        if let Some(mut prev) = self.0.take(&MemRangeVma(MemRange::new(
            (range.start() - 1.into()).into(),
            1,
        ))) {
            prev.size += range.size;
            self.insert(prev)
        } else if let Some(mut next) = self.0.take(&MemRangeVma(MemRange::new(
            (range.start() + VirtAddr::new(1)).into(),
            1,
        ))) {
            next.start = range.start;
            next.size += range.size;
            self.insert(next)
        } else {
            self.0.insert(range);
        }
    }
}

pub struct VmaList {
    occupied: BTreeSet<Vma>,
    free: FreeRegions,
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

            let vma_middle = Self::new_fixed(addr, size);

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

    pub fn new_user() -> Self {
        Self {
            free: FreeRegions(BTreeSet::from([MemRangeVma::max_user()])),
            occupied: BTreeSet::new(),
        }
    }

    pub fn new_kernel() -> Self {
        Self {
            free: FreeRegions(BTreeSet::from([MemRangeVma::max_kernel()])),
            occupied: BTreeSet::new(),
        }
    }

    pub fn add_to_tree(&mut self, vma: Vma) -> Result<VirtAddr, ()> {
        let mut vma = vma;

        let range = if !vma.is_fixed() {
            let range = self.find_free_range(vma.size(), None).ok_or(())?;

            // Update range with new address
            vma.range = MemRangeVma(MemRange {
                start: range.start(),
                size: vma.size(),
            });

            range
        } else {
            self.find_free_range(vma.size(), Some(vma.range.0.start))
                .ok_or(())?
        };

        let splitted = range.split_at(vma.start(), vma.size());

        for i in splitted {
            if i.start() == vma.start() {
                assert!(self.occupied.insert(vma.clone()));
            } else if i.is_valid() {
                assert!(!i.start().is_null());
                self.free.insert(i);
            }
        }

        Ok(vma.start())
    }

    pub fn free(&mut self, range: MemRange<VirtAddr>) -> Result<(), ErrorType> {
        let vma = self
            .occupied
            .take(&Vma::from_range(range))
            .ok_or(ErrorType::NotFound)?;

        if vma.range.0 != range {
            let split = vma.range.split_at(range.start(), range.size());

            for i in split {
                if i.start() == vma.start() {
                    self.free.insert(vma.range.clone());
                } else if i.is_valid() {
                    self.occupied.insert(Vma {
                        range: i,
                        tp: vma.tp,
                        state: vma.state,
                    });
                }
            }
        }

        Ok(())
    }

    fn find_free_range(&mut self, size: usize, addr: Option<VirtAddr>) -> Option<MemRangeVma> {
        if let Some(addr) = addr {
            self.free.find_by_addr(addr)
        } else {
            self.free.find_by_size(size)
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
        Self(super::layout::vmm_range(
            loader_protocol::VmmLayoutKind::User,
        ))
    }

    pub fn max_kernel() -> Self {
        Self(super::layout::vmm_range(
            loader_protocol::VmmLayoutKind::VmAlloc,
        ))
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
        let mut list = VmaList::new_user();

        test_assert!(list.find_free_range(PAGE_SIZE, None).is_some());
    }

    #[kernel_test]
    fn vma_list_add() {
        let mut list = VmaList::new_user();
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
        let mut list = VmaList::new_user();

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
        let mut list = VmaList::new_user();
        let vma = Vma::new(
            MemRangeVma::new_fixed(0x2000.into(), 0x1000),
            MappingType::USER_DATA,
        );

        let va = list.add_to_tree(vma.clone());

        test_assert!(va.is_ok());
        test_assert!(list.add_to_tree(vma.clone()).is_err());

        list.free(vma.range.0.clone());
        test_assert_eq!(list.vma_list_sorted().len(), 1);

        test_assert!(list.add_to_tree(vma).is_ok());
    }
}
