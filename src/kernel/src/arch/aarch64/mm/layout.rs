use crate::mm::layout::LayoutEntry;
use rtl::vmm::types::{MemRange, VirtAddr};
use rtl::vmm::types::PhysAddr;
use spin::Once;

static LOAD_ADDR: Once<PhysAddr> = Once::new();

pub const VMM_LAYOUT: [MemRange<VirtAddr>; LayoutEntry::Count as usize] = [
    MemRange::new(VirtAddr::new(0xffffffa000000000), 20 << 30),
    MemRange::new(VirtAddr::new(0xffffffa500000000), 1 << 30),
    MemRange::new(VirtAddr::new(0xffffffa540000000), 1 << 30),
    MemRange::new(VirtAddr::new(0xffffffa580000000), 30 << 30),
];

pub fn init(load_addr: PhysAddr) {
    LOAD_ADDR.call_once(|| load_addr);
}

pub fn image_to_phys(addr: VirtAddr) -> PhysAddr {
    let base = VMM_LAYOUT[LayoutEntry::Image as usize].start();
    let diff = addr - base;

    unsafe { *LOAD_ADDR.get_unchecked() + PhysAddr::new(diff) }
}
