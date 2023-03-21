pub mod allocators;
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
    PhysAddr::from(virt.bits() - kernel_offset())
}

pub fn init() {
    allocators::boot_alloc::init();
    allocators::page_alloc::init();
    paging::kernel_page_table::init();
    sections::remap_kernel();
    crate::kernel::percpu::init_percpu();
    allocators::stack_alloc::init_kernel_stacks();
}
