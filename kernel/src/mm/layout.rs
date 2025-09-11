use crate::arch::mm::layout::VMM_LAYOUT;
use rtl::vmm::types::{VirtAddr, MemRange};

#[repr(usize)]
pub enum LayoutEntry {
    Image = 0,
    Mmio = 1,
    Fixmap = 2,
    VmAlloc = 3,
    Count = 4,
}

pub fn vmm_range(e: LayoutEntry) -> MemRange<VirtAddr> {
    // VMM_LAYOUT[e as usize]
    todo!()
}
