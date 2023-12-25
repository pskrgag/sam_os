use alloc::rc::Rc;
use core::cmp::Ordering;
use intrusive_collections::{
    intrusive_adapter,
    rbtree::CursorMut,
    KeyAdapter, RBTree, RBTreeLink,
};
use shared::vmm::MappingType;
use shared::vmm::types::*;

#[derive(Debug, Eq, Clone, Copy)]
pub(crate) struct MemRangeVma(MemRange<VirtAddr>);

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum VmaFlags {
    VmaFixed,
    VmaFree,
    VmaCommited,
}

#[derive(Debug)]
pub struct Vma {
    pub(crate) range: MemRangeVma,
    pub(crate) tp: MappingType,
    pub(crate) state: VmaFlags,
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

impl VmaList {
    pub fn dump_tree(&self) {
        for i in &self.list {
            println!("{:?}", i);
        }
    }

    pub fn new() -> Self {
        let mut list = RBTree::new(VmaAdapter::new());

        list.insert(Rc::new(Vma::default_user()));
        Self { list }
    }

    fn find_free_vma_addr(&mut self, addr: VirtAddr) -> CursorMut<'_, VmaAdapter> {
        self.list.find_mut(&MemRangeVma(MemRange::new(addr, 1)))
    }

    pub fn add_to_tree(&mut self, vma: Vma) -> Result<VirtAddr, ()> {
        let addr = vma.start();
        let size = vma.size();

        let mut vma_c = self.find_free_vma_addr(vma.start());

        if vma_c.get().unwrap().is_free() {
            let mut vmas: [Vma; 2] = core::array::from_fn(|_| Vma::default_user());
            let mut vma = vma_c.remove().unwrap();
            let flags = vma.map_flags();

            let c = Rc::get_mut(&mut vma)
                .unwrap()
                .split_at(addr, size, flags, &mut vmas);

            for (cnt, i) in vmas.into_iter().enumerate() {
                if cnt >= c {
                    break;
                }

                self.list.insert(Rc::new(i));
            }

            self.list.insert(vma);
            Ok(addr)
        } else {
            Err(())
        }
    }

    pub fn free_range(&self, size: usize) -> Option<MemRange<VirtAddr>> {
        for i in &self.list {
            if i.is_free() && i.size() >= size {
                return Some(MemRange::new(i.start(), size));
            }
        }

        None
    }

    pub fn free_range_at(&self, range: MemRange<VirtAddr>) -> Option<MemRange<VirtAddr>> {
        for i in &self.list {
            if i.is_free() && i.contains_range(range) {
                return Some(range);
            }
        }

        None
    }
}

impl Vma {
    pub fn new(range: MemRange<VirtAddr>, tp: MappingType) -> Self {
        assert!(range.start().is_page_aligned());

        Self {
            range: MemRangeVma(range),
            tp,
            state: VmaFlags::VmaFree,
            non_free_link: RBTreeLink::new(),
        }
    }

    pub fn mark_allocated(&mut self) {
        self.state = VmaFlags::VmaCommited;
    }

    pub fn split_at(
        &mut self,
        addr: VirtAddr,
        size: usize,
        tp: MappingType,
        out: &mut [Vma],
    ) -> usize {
        let range = &self.range.0;
        let start = range.start();
        let end = start + range.size();

        if addr != range.start() && addr.bits() != range.start() + range.size() - size {
            self.range = MemRangeVma(MemRange::new(start, addr - start));

            out[0] = Vma::new(MemRange::new(addr, size), tp);
            out[1] = Vma::new(
                MemRange::new(VirtAddr::new(addr + size), end - addr.bits() + size),
                self.tp,
            );

            out[0].mark_allocated();

            2
        } else if addr == range.start() {
            out[0] = Vma::new(MemRange::new(addr, size), tp);

            self.range = MemRangeVma(MemRange::new(
                VirtAddr::new(addr + size),
                range.size() - size,
            ));

            out[0].mark_allocated();
            1
        } else {
            self.range = MemRangeVma(MemRange::new(range.start(), range.size() - size));

            out[0] = Vma::new(MemRange::new(addr, size), tp);
            out[0].mark_allocated();
            1
        }
    }

    pub fn is_free(&self) -> bool {
        self.state == VmaFlags::VmaFree
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

    pub fn default_user() -> Vma {
        Self::new(MemRange::max_user(), MappingType::NONE)
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
