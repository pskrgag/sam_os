pub mod allocator;
pub mod boot_alloc;
pub mod page_alloc;
pub mod paging;
pub mod sections;
pub mod types;

use crate::kernel::misc::kernel_offset;
use types::*;

#[inline]
pub fn phys_to_virt_linear(phys: PhysAddr) -> VirtAddr {
    VirtAddr::from(phys.get() + kernel_offset())
}

#[inline]
pub fn virt_to_phys_linear(virt: VirtAddr) -> PhysAddr {
    PhysAddr::from(virt.get() - kernel_offset())
}
