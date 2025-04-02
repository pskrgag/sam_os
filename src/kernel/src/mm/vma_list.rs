use alloc::rc::Rc;
use core::cmp::Ordering;
use intrusive_collections::{intrusive_adapter, rbtree::CursorMut, KeyAdapter, RBTree, RBTreeLink};
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

#[derive(Debug, Eq, Clone, Copy)]
pub(crate) struct MemRangeVma(MemRange<VirtAddr>);

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum VmaFlags {
    VmaFree,
    VmaInvalid,
    VmaAllocated,
}

#[derive(Debug, Clone)]
pub struct Vma {
    pub(crate) range: MemRangeVma,
    pub(crate) tp: MappingType,
    state: VmaFlags,
    pub(crate) non_free_link: RBTreeLink,
}

pub struct VmaList {
    list: RBTree<VmaAdapter>,
}

intrusive_adapter!(VmaAdapter = Rc<Vma>: Vma { non_free_link: RBTreeLink });

impl<'a> KeyAdapter<'a> for VmaAdapter {
    type Key = MemRangeVma;

    fn get_key(&self, x: &'a Vma) -> Self::Key {
        x.range
    }
}

// TODO: all these APIs are pure garbage and need rework one day
impl VmaList {
    #[cfg(test)]
    pub fn vma_list_sorted(&self) -> alloc::vec::Vec<Vma> {
        self.list.iter().map(|x| x.clone()).collect()
    }

    pub fn new() -> Self {
        let mut list = RBTree::new(VmaAdapter::new());

        list.insert(Rc::new(Vma::default_user()));
        Self { list }
    }

    fn find_vma_addr(&mut self, addr: VirtAddr) -> CursorMut<'_, VmaAdapter> {
        self.list.find_mut(&MemRangeVma(MemRange::new(addr, 1)))
    }

    pub fn add_to_tree(&mut self, vma: Vma) -> Result<VirtAddr, ()> {
        let addr = if !vma.is_fixed() {
            vma.start()
        } else {
            self.free_range(vma.size()).ok_or(())?.start()
        };
        let size = vma.size();

        let mut vma_c = self.find_vma_addr(addr);

        // 1. Get VMA that holds an address
        if let Some(vma) = vma_c.get() {
            // 2. Check that VMA is free
            if !vma.is_free() {
                return Err(());
            }

            // 3. Split if needed
            if vma.size() != size {
                let flags = vma.map_flags();
                let clone = vma_c.remove().unwrap();
                let arr = Rc::into_inner(clone).unwrap().split_at(addr, size, flags);

                for i in arr.into_iter() {
                    if !i.is_valid() {
                        break;
                    }

                    self.list.insert(Rc::new(i));
                }
            } else {
                let mut allocated = vma_c.get().unwrap().clone();

                allocated.mark_allocated();
                vma_c.replace_with(Rc::new(allocated)).unwrap();
            }

            Ok(addr)
        } else {
            Err(())
        }
    }

    pub fn mark_free(&mut self, vma: Vma) -> Option<()> {
        let mut vma_c = self.find_vma_addr(vma.start());

        if let Some(vma) = vma_c.get() {
            let mut v = vma.clone();
            v.mark_free();

            vma_c.move_next();

            if let Some(next) = vma_c.get() {
                if next.is_free() {
                    let next = vma_c.remove().unwrap();
                    v.merge(Rc::into_inner(next).unwrap()).unwrap();
                }
            }

            vma_c.move_prev();
            vma_c.move_prev();

            if let Some(prev) = vma_c.get() {
                if prev.is_free() {
                    let prev = vma_c.remove().unwrap();
                    v.merge(Rc::into_inner(prev).unwrap()).unwrap();
                }
            }

            vma_c.replace_with(Rc::new(v)).unwrap();
            Some(())
        } else {
            None
        }
    }

    fn free_range(&self, size: usize) -> Option<MemRange<VirtAddr>> {
        for i in &self.list {
            if i.is_free() && i.size() >= size {
                return Some(MemRange::new(i.start(), size));
            }
        }

        None
    }
}

impl Vma {
    pub fn new(range: MemRangeVma, tp: MappingType) -> Self {
        assert!(range.0.start().is_page_aligned());

        Self {
            range,
            tp,
            state: VmaFlags::VmaFree,
            non_free_link: RBTreeLink::new(),
        }
    }

    pub fn default_user() -> Self {
        Self::new(MemRangeVma::max_user(), MappingType::NONE)
    }

    pub fn invalid() -> Self {
        Self {
            range: MemRangeVma::new_fixed(0.into(), 0),
            tp: MappingType::USER_DEVICE,
            state: VmaFlags::VmaInvalid,
            non_free_link: RBTreeLink::new(),
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

    pub fn split_at(mut self, addr: VirtAddr, size: usize, tp: MappingType) -> [Vma; 3] {
        let range = self.range.0;
        let start = range.start();
        let isize = range.size();

        // Split at 3
        if addr != range.start() && addr.bits() != range.start() + range.size() - size {
            self.range = MemRangeVma(MemRange::new(start, addr - start));

            let mut vma_middle = Vma::new(MemRangeVma::new_fixed(addr, size), tp);

            let vma_higer = Vma::new(
                MemRangeVma::new_fixed(
                    VirtAddr::new(addr + size),
                    isize - self.size() - vma_middle.size(),
                ),
                self.tp,
            );

            vma_middle.mark_allocated();

            [self, vma_middle, vma_higer]
        } else if addr == range.start() {
            let mut vma_lower = Vma::new(MemRangeVma::new_fixed(addr, size), tp);

            self.range = MemRangeVma::new_fixed(VirtAddr::new(addr + size), range.size() - size);

            vma_lower.mark_allocated();
            [vma_lower, self, Vma::invalid()]
        } else {
            self.range = MemRangeVma::new_fixed(range.start(), range.size() - size);

            let mut vma_higer = Vma::new(MemRangeVma::new_fixed(addr, size), tp);
            vma_higer.mark_allocated();
            [self, vma_higer, Vma::invalid()]
        }
    }

    pub fn mark_allocated(&mut self) {
        self.state = VmaFlags::VmaAllocated;
    }

    pub fn is_free(&self) -> bool {
        self.state == VmaFlags::VmaFree
    }

    pub fn is_valid(&self) -> bool {
        self.state != VmaFlags::VmaInvalid
    }

    pub fn mark_free(&mut self) {
        self.state = VmaFlags::VmaFree;
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
        self.range.0.start().is_null()
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
        if self.0.contains_addr(other.0.start()) {
            Some(Ordering::Equal)
        } else {
            Some(self.0.cmp(&other.0))
        }
    }
}

impl Ord for MemRangeVma {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0.contains_addr(other.0.start()) {
            Ordering::Equal
        } else if other.0.contains_addr(self.0.start()) {
            Ordering::Equal
        } else {
            self.0.cmp(&other.0)
        }
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

        list.mark_free(vma.clone());
        test_assert_eq!(list.vma_list_sorted().len(), 1);

        test_assert!(list.add_to_tree(vma).is_ok());
    }
}
