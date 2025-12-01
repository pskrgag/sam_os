use alloc::boxed::Box;
use core::cmp::Ordering;
use core::ops::{Bound, Deref, DerefMut, RangeBounds};
use core::pin::Pin;
use core::ptr::NonNull;
use hal::address::*;
use rtl::vmm::MappingType;
use wavltree::{Linked, Links, Side, WAVLTree};

#[derive(Default, Debug, Clone)]
struct NodeState {
    min_byte: VirtAddr,
    max_byte: VirtAddr,
    max_gap: usize,
}

struct Vma {
    links: Links<Self>,
    range: MemRangeVma,
    flags: MappingType,
    stats: NodeState,
}

impl core::fmt::Debug for Vma {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Range {:?}, MaxGap: {:x} Max Byte: {:x}",
            self.range.0, self.stats.max_gap, self.stats.max_byte
        )
    }
}

unsafe impl Linked for Vma {
    type Handle = Pin<Box<Self>>;
    type Key = MemRangeVma;

    fn into_ptr(handle: Self::Handle) -> NonNull<Self> {
        unsafe { NonNull::from(Box::leak(Pin::into_inner_unchecked(handle))) }
    }

    unsafe fn from_ptr(ptr: NonNull<Self>) -> Self::Handle {
        unsafe { Pin::new_unchecked(Box::from_raw(ptr.as_ptr())) }
    }

    unsafe fn links(ptr: NonNull<Self>) -> NonNull<wavltree::Links<Self>> {
        ptr.map_addr(|addr| {
            let offset = core::mem::offset_of!(Self, links);
            addr.checked_add(offset).unwrap()
        })
        .cast()
    }

    fn get_key(&self) -> &Self::Key {
        &self.range
    }

    fn after_insert(self: Pin<&mut Self>) {
        unsafe {
            let vma = Pin::into_inner_unchecked(self);

            vma.stats.min_byte = vma.first_occupied_byte();
            vma.stats.max_byte = vma.last_occupied_byte();
            vma.stats.max_gap = 0;

            vma.propogate_stats_to_root();
        }
    }

    fn after_rotate(
        self: Pin<&mut Self>,
        mut parent: NonNull<Self>,
        sibling: Option<NonNull<Self>>,
        lr_child: Option<NonNull<Self>>,
        side: Side,
    ) {
        unsafe {
            let pivot = Pin::into_inner_unchecked(self);
            let parent = parent.as_mut();

            pivot.stats = parent.stats.clone();

            match side {
                Side::Left => parent.update_state(sibling, lr_child),
                Side::Right => parent.update_state(lr_child, sibling),
            }
        }
    }
}

impl Vma {
    pub fn new(start: VirtAddr, size: usize, flags: MappingType) -> Self {
        Self {
            links: Links::new(),
            range: MemRangeVma(MemRange {
                start: start.into(),
                size,
            }),
            stats: NodeState::default(),
            flags,
        }
    }

    pub fn min_byte(&self) -> VirtAddr {
        self.stats.min_byte
    }

    pub fn max_byte(&self) -> VirtAddr {
        self.stats.max_byte
    }

    pub fn max_gap(&self) -> usize {
        self.stats.max_gap
    }

    pub fn first_occupied_byte(&self) -> VirtAddr {
        self.range.start
    }

    pub fn last_occupied_byte(&self) -> VirtAddr {
        (self.range.start + self.range.size - 1).into()
    }

    fn propogate_stats_to_root(&mut self) {
        let mut iter = self.links.parent();

        while let Some(mut vma) = iter {
            let vma = unsafe { vma.as_mut() };

            vma.update_state(vma.links.left(), vma.links.right());
            iter = vma.links.parent();
        }
    }

    fn update_state(&mut self, left: Option<NonNull<Vma>>, right: Option<NonNull<Vma>>) {
        let left_gap = if let Some(left) = left {
            let left = unsafe { left.as_ref() };
            let gap_left = self.first_occupied_byte().bits() - left.max_byte().bits() - 1;

            self.stats.min_byte = left.stats.min_byte;
            gap_left.max(left.max_gap())
        } else {
            self.stats.min_byte = self.first_occupied_byte();
            0
        };

        let right_gap = if let Some(right) = right {
            let right = unsafe { right.as_ref() };
            let gap_right = right.min_byte() - self.last_occupied_byte() - 1;

            self.stats.max_byte = right.stats.max_byte;
            gap_right.max(right.max_gap())
        } else {
            self.stats.max_byte = self.last_occupied_byte();
            0
        };

        self.stats.max_gap = left_gap.max(right_gap);
    }
}

#[derive(Debug, Eq, Clone, Copy)]
struct MemRangeVma(MemRange<VirtAddr>);

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

pub struct VmaList {
    tree: WAVLTree<Vma>,
    start: usize,
    size: usize,
}

impl MemRangeVma {
    pub fn new(start: usize, size: usize) -> Self {
        Self(MemRange {
            start: start.into(),
            size,
        })
    }

    // pub fn invalid() -> Self {
    //     Self(MemRange::new(VirtAddr::from(usize::MAX), 0))
    // }
    //
    // pub fn is_valid(&self) -> bool {
    //     self.start() != usize::MAX.into()
    // }
    //
    // pub fn split_at(mut self, addr: VirtAddr, size: usize) -> [MemRangeVma; 3] {
    //     let range = self.0;
    //     let start = range.start();
    //     let isize = range.size();
    //
    //     assert!(self.0.contains_addr(addr));
    //     assert!(self != Self::invalid());
    //
    //     // Split at 3
    //     if addr != range.start() && addr.bits() != range.start() + range.size() - size {
    //         self = Self(MemRange::new(start, addr - start));
    //
    //         let vma_middle = Self::new_fixed(addr, size);
    //
    //         let vma_higer = MemRangeVma::new_fixed(
    //             VirtAddr::new(addr + size),
    //             isize - self.0.size() - vma_middle.0.size(),
    //         );
    //
    //         [self, vma_middle, vma_higer]
    //     } else if addr == range.start() {
    //         let vma_lower = MemRangeVma::new_fixed(addr, size);
    //
    //         self = MemRangeVma::new_fixed(VirtAddr::new(addr + size), range.size() - size);
    //
    //         [vma_lower, self, Self::invalid()]
    //     } else {
    //         self = MemRangeVma::new_fixed(range.start(), range.size() - size);
    //
    //         let vma_higer = MemRangeVma::new_fixed(addr, size);
    //         [self, vma_higer, Self::invalid()]
    //     }
    // }
}

impl VmaList {
    pub fn new_user() -> Self {
        let range = super::layout::vmm_range(loader_protocol::VmmLayoutKind::User);

        Self {
            tree: WAVLTree::new(),
            start: range.start.into(),
            size: range.size(),
        }
    }

    pub fn new_kernel() -> Self {
        let range = super::layout::vmm_range(loader_protocol::VmmLayoutKind::VmAlloc);

        Self {
            tree: WAVLTree::new(),
            start: range.start.into(),
            size: range.size,
        }
    }

    fn gap_left(&self, vma: &Vma) -> usize {
        let left = vma.links.left();

        if let Some(left) = left {
            let left = unsafe { left.as_ref() };

            vma.first_occupied_byte() - left.max_byte() + 1
        } else {
            0
        }
    }

    fn gap_right(&self, vma: &Vma) -> usize {
        let right = vma.links.right();

        if let Some(right) = right {
            unsafe { right.as_ref().min_byte() - vma.last_occupied_byte() + 1 }
        } else {
            0
        }
    }

    fn find_free_space(&self, size: usize, base: Option<usize>) -> Option<VirtAddr> {
        if size > self.size {
            return None;
        }

        if self.tree.size() == 0 {
            // If tree is empty just take the address from the beginning.
            let start = base.unwrap_or(self.start);

            Some(start.into())
        } else if let Some(base) = base {
            if base < self.start {
                return None;
            }

            // Find the lower bound for the address
            let cursor = self
                .tree
                .upper_bound(Bound::Included(&MemRangeVma::new(base, size)));

            // If lower bound exists, check if it contains specified range
            if let Some(vma) = cursor.get() {
                let right = vma.links.right();

                if vma.range.contains_addr(base.into()) {
                    return None;
                }

                let enough_space = if let Some(right) = right {
                    unsafe { right.as_ref().range.start - base.into() >= size }
                } else {
                    self.start + self.size - base >= size
                };

                enough_space.then_some(base.into())
            } else {
                // If it does not exists, check the range from the start of the address space to
                // the base
                let root = self.tree.root().get().expect("Root must exist here");

                (root.min_byte() - base.into() > size).then_some(base.into())
            }
        } else {
            // User wants to find any free rang with size.
            let root = self.tree.root().get().expect("Root must exist here");

            if root.max_gap() == 0 {
                // There is no gaps in current sub-tree. It's can be because of
                //
                // 1) Tree is full
                // 2) No gaps between allocated VMAs

                // Tree is full
                if root.max_byte().bits() == self.start + self.size - 1
                    && root.min_byte().bits() == self.start
                {
                    return None;
                }

                if root.min_byte().bits() != self.start {
                    let gap = root.min_byte().bits() - self.start;

                    (gap >= size).then_some(self.start.into())
                } else if root.max_byte().bits() != self.start + self.size - 1 {
                    let space_end = self.start + self.size - 1;
                    let gap = space_end - root.max_byte().bits();

                    (gap >= size).then_some((root.max_byte() + 1).into())
                } else {
                    None
                }
            } else {
                let can_insert_here = |vma: &Vma, size: usize| -> Option<VirtAddr> {
                    unsafe {
                        if self.gap_left(vma) >= size {
                            Some((vma.links.left().unwrap().as_ref().max_byte() + 1).into())
                        } else if self.gap_right(vma) >= size {
                            Some((vma.last_occupied_byte() + 1).into())
                        } else {
                            None
                        }
                    }
                };

                let mut root = self.tree.root().get();

                while let Some(vma) = root {
                    if let Some(start) = can_insert_here(vma, size) {
                        return Some(start);
                    }

                    if vma.links.left().is_none() && vma.links.right().is_none() {
                        break;
                    }

                    unsafe {
                        if let Some(left) = vma.links.left() && left.as_ref().max_gap() >= size {
                            root = Some(left.as_ref());
                            continue;
                        }
                    }

                    unsafe {
                        if let Some(right) = vma.links.right() && right.as_ref().max_gap() >= size {
                            root = Some(right.as_ref());
                            continue;
                        }
                    }

                    break;
                }

                // We can only insert at the end or at the beginning here

                let root = root.unwrap();

                if root.min_byte().bits() - self.start >= size {
                    return Some(self.start.into());
                }

                if self.start + self.size - root.max_byte().bits() - 1 >= size {
                    return Some((root.max_byte() + 1).into());
                }

                None
            }
        }
    }

    pub fn new_vma(
        &mut self,
        size: usize,
        base: Option<usize>,
        mt: MappingType,
    ) -> Option<VirtAddr> {
        let start = self.find_free_space(size, base)?;
        let vma = Box::try_new(Vma::new(start, size, mt)).ok()?;

        debug_assert!(start.is_page_aligned());

        self.tree.insert(vma.into());
        Some(start)
    }

    pub fn free(&mut self, _range: MemRange<VirtAddr>) {
        if self.tree.size() == 0 {
            return;
        }

        todo!()
    }
}

impl From<MemRange<VirtAddr>> for MemRangeVma {
    fn from(value: MemRange<VirtAddr>) -> Self {
        Self(value)
    }
}

impl PartialEq for MemRangeVma {
    fn eq(&self, other: &Self) -> bool {
        self.0.start == other.0.start
    }
}

impl PartialOrd for MemRangeVma {
    fn partial_cmp(&self, other: &MemRangeVma) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemRangeVma {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.start.cmp(&other.start)
    }
}

impl PartialEq for Vma {
    fn eq(&self, other: &Self) -> bool {
        self.range.eq(&other.range)
    }
}

impl PartialOrd for Vma {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Vma {
    fn cmp(&self, other: &Self) -> Ordering {
        self.range.cmp(&other.range)
    }
}

impl Eq for Vma {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use test_macros::*;

    #[kernel_test]
    fn vma_list_add_fixed() {
        let mut list = VmaList::new_user();

        let new_vma = list.new_vma(0x1000, Some(0x20000), MappingType::None);
        test_assert_eq!(new_vma, Some(VirtAddr::new(0x20000)));

        let new_vma = list.new_vma(0x1000, Some(0x30000), MappingType::None);
        test_assert_eq!(new_vma, Some(VirtAddr::new(0x30000)));

        let new_vma = list.new_vma(0x1000, Some(0x1000), MappingType::None);
        test_assert!(new_vma.is_none());

        let new_vma = list.new_vma(0x1000, Some(0x0), MappingType::None);
        test_assert!(new_vma.is_none());
    }
}
