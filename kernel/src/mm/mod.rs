use loader_protocol::LoaderArg;
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::{PhysAddr, VirtAddr};

pub mod allocators;
pub mod layout;
pub mod paging;
pub mod user_buffer;
pub mod vma_list;
pub mod vms;

pub unsafe fn memset_pages(pa: PhysAddr, num: usize) {
    let mut va = VirtAddr::from(pa);

    unsafe { va.as_slice_mut::<u8>(num * PAGE_SIZE).fill(0x00) };
}

pub fn init(prot: &LoaderArg) {
    layout::init(prot);
    allocators::page_alloc::init(prot);
    paging::kernel_page_table::init(prot);
    allocators::slab::init_kernel_slabs();
}
