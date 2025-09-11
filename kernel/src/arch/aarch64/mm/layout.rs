use crate::mm::layout::LayoutEntry;
use rtl::vmm::types::{MemRange, VirtAddr};

pub const VMM_LAYOUT: [MemRange<VirtAddr>; LayoutEntry::Count as usize] = [
    MemRange::new(VirtAddr::new(0xffffffa000000000), 20 << 30),
    MemRange::new(VirtAddr::new(0xffffffa500000000), 1 << 30),
    MemRange::new(VirtAddr::new(0xffffffa540000000), 1 << 30),
    MemRange::new(VirtAddr::new(0xffffffa580000000), 30 << 30),
];
