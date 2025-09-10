use super::regions::regions;
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::{Address, PhysAddr};

pub fn alloc_pages(count: usize) -> Option<PhysAddr> {
    for reg in regions() {
        if reg.count > count {
            let addr = reg.start;

            reg.start = reg.start + PhysAddr::new(count * PAGE_SIZE);
            reg.count -= count;

            unsafe { core::slice::from_raw_parts_mut(addr.bits() as *mut u8, PAGE_SIZE).fill(0x0) };
            return Some(addr);
        }
    }

    None
}
