use crate::{
    mm::{
        types::{VirtAddr, MemRange},
        paging::page_table::MappingType,
    },
};
use alloc::collections::LinkedList;

struct Vma {
    pub range: MemRange<VirtAddr>,
    pub tp: MappingType,
}

pub struct VmaList {
    list: LinkedList<Vma>,
}

#[inline]
fn do_intersect(range1: &MemRange<VirtAddr>, range2: &MemRange<VirtAddr>) -> bool {
    (range1.start() + range1.size()) < range2.start().into() &&
        (range2.start() + range2.size()) < range1.start().into()
}

impl VmaList {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    pub fn add(&mut self, range: MemRange<VirtAddr>, tp: MappingType) {
        let vma = Vma::new(range, tp);
        let mut cursor = self.list.cursor_front_mut();

        while let Some(i) = cursor.current() {
            if i.range.start() < vma.range.start() {
                if do_intersect(&i.range, &vma.range) {
                    // VMA merge
                    todo!()
                } else {
                    cursor.insert_before(vma);
                    break;
                }
            }

            cursor.move_next();
        }
    }

    // TODO: redo
    pub fn free_range(&self, size: usize, start: VirtAddr, vms_size: usize) -> Option<MemRange<VirtAddr>> {
        if let Some(back) = self.list.back() {
            if back.range.start() + back.range.size() < start.get() + vms_size  - size - 1 {
                Some(MemRange::new(VirtAddr::from(back.range.start() + back.range.size()), size))
            } else {
                todo!();
            }
        } else {
            Some(MemRange::new(start, size))
        }
    }
}

impl Vma {
    pub fn new(range: MemRange<VirtAddr>, tp: MappingType) -> Self {
        Self {
            range: range,
            tp: tp,
        }
    }

}
