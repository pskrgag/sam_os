use hal::address::{LinearAddr, PhysAddr, VirtAddr};
use hal::arch::PAGE_SIZE;
use loader_protocol::LoaderArg;

pub mod allocators;
pub mod paging;
pub mod user_buffer;
pub mod vmm;

pub unsafe fn memset_pages(pa: PhysAddr, num: usize) {
    let linear = LinearAddr::from(pa);
    let mut va: VirtAddr = linear.into();

    unsafe { va.as_slice_mut::<u8>(num * PAGE_SIZE).fill(0x00) };
}

pub fn init(prot: &LoaderArg) {
    vmm::init(prot);
    allocators::page_alloc::init(prot);
    paging::kernel_page_table::init(prot);
    allocators::slab::init_kernel_slabs();
}
