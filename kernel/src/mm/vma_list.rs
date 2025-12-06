use alloc::boxed::Box;
use core::cmp::Ordering;
use core::ops::Bound;
use core::pin::Pin;
use core::ptr::NonNull;
use hal::address::*;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;
use wavltree::{Linked, Links, Side, WAVLTree};

#[derive(Default, Debug, Clone)]
struct NodeState {
    min_byte: VirtAddr,
    max_byte: VirtAddr,
    max_gap: usize,
}

use bitmask::bitmask;

bitmask! {
    pub mask VmaFlags: u8 where flags VmaFlag {
        None = 0,
        ExternalPages = 1,
    }
}

struct Vma {
    links: Links<Self>,
    range: MemRange<VirtAddr>,
    prot: MappingType,
    flags: VmaFlags,
    stats: NodeState,
}

impl core::fmt::Debug for Vma {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Range {:?}, MaxGap: {:x} Max Byte: {:x}",
            self.range, self.stats.max_gap, self.stats.max_byte
        )
    }
}

unsafe impl Linked for Vma {
    type Handle = Pin<Box<Self>>;
    type Key = VirtAddr;

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
        &self.range.start
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
    pub fn new(start: VirtAddr, size: usize, prot: MappingType, flags: VmaFlags) -> Self {
        Self {
            links: Links::new(),
            range: MemRange {
                start: start.into(),
                size,
            },
            stats: NodeState::default(),
            prot,
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

pub struct VmaList {
    tree: WAVLTree<Vma>,
    start: usize,
    size: usize,
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

    fn find_free_space(&self, size: usize, base: Option<usize>) -> Result<VirtAddr, ErrorType> {
        if size > self.size {
            return Err(ErrorType::InvalidArgument);
        }

        if self.tree.size() == 0 {
            // If tree is empty just take the address from the beginning.
            let start = base.unwrap_or(self.start);

            Ok(start.into())
        } else if let Some(base) = base {
            if base < self.start {
                return Err(ErrorType::InvalidArgument);
            }

            // Find the lower bound for the address
            let cursor = self.tree.upper_bound(Bound::Included(&VirtAddr::new(base)));

            // If lower bound exists, check if it contains specified range
            if let Some(vma) = cursor.get() {
                let right = vma.links.right();

                if vma.range.contains_addr(base.into()) {
                    return Err(ErrorType::AlreadyExists);
                }

                let enough_space = if let Some(right) = right {
                    unsafe { right.as_ref().range.start - base.into() >= size }
                } else {
                    self.start + self.size - base >= size
                };

                enough_space
                    .then_some(base.into())
                    .ok_or(ErrorType::NoMemory)
            } else {
                // If it does not exists, check the range from the start of the address space to
                // the base
                let root = self.tree.root().get().expect("Root must exist here");

                (root.min_byte() - base.into() > size)
                    .then_some(base.into())
                    .ok_or(ErrorType::NoMemory)
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
                    return Err(ErrorType::NoMemory);
                }

                if root.min_byte().bits() != self.start {
                    let gap = root.min_byte().bits() - self.start;

                    (gap >= size)
                        .then_some(self.start.into())
                        .ok_or(ErrorType::NoMemory)
                } else if root.max_byte().bits() != self.start + self.size - 1 {
                    let space_end = self.start + self.size - 1;
                    let gap = space_end - root.max_byte().bits();

                    (gap >= size)
                        .then_some((root.max_byte() + 1).into())
                        .ok_or(ErrorType::NoMemory)
                } else {
                    Err(ErrorType::NoMemory)
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
                        return Ok(start);
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
                    return Ok(self.start.into());
                }

                if self.start + self.size - root.max_byte().bits() >= size + 1 {
                    return Ok((root.max_byte() + 1).into());
                }

                Err(ErrorType::NoMemory)
            }
        }
    }

    pub fn new_vma(
        &mut self,
        size: usize,
        base: Option<usize>,
        mt: MappingType,
        flags: VmaFlags,
    ) -> Result<VirtAddr, ErrorType> {
        let start = self.find_free_space(size, base)?;
        let vma =
            Box::try_new(Vma::new(start, size, mt, flags)).map_err(|_| ErrorType::NoMemory)?;

        debug_assert!(start.is_page_aligned());

        self.tree.insert(vma.into());
        Ok(start)
    }

    pub fn free(&mut self, _range: MemRange<VirtAddr>) -> Result<(), ErrorType> {
        if self.tree.size() == 0 {
            return Ok(());
        }

        todo!()
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
        self.range.start().cmp(&other.range.start)
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

        let new_vma = list.new_vma(
            0x1000,
            Some(0x20000),
            MappingType::None,
            VmaFlag::None.into(),
        );
        test_assert_eq!(new_vma, Some(VirtAddr::new(0x20000)));

        let new_vma = list.new_vma(
            0x1000,
            Some(0x30000),
            MappingType::None,
            VmaFlag::None.into(),
        );
        test_assert_eq!(new_vma, Some(VirtAddr::new(0x30000)));

        let new_vma = list.new_vma(
            0x1000,
            Some(0x1000),
            MappingType::None,
            VmaFlag::None.into(),
        );
        test_assert!(new_vma.is_none());

        let new_vma = list.new_vma(0x1000, Some(0x0), MappingType::None, VmaFlag::None.into());
        test_assert!(new_vma.is_none());
    }
}
