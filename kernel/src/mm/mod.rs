use hal::address::{LinearAddr, Pfn, PhysAddr, VirtualAddress};
use hal::arch::PAGE_SIZE;
use loader_protocol::LoaderArg;

pub mod allocators;
pub mod paging;
pub mod pmm;
pub mod user_buffer;
pub mod vmm;

pub unsafe fn memset_pages(pfn: Pfn, num: usize) {
    let pa: PhysAddr = pfn.into();
    let mut va = LinearAddr::from(pa);

    unsafe { va.as_slice_mut::<u8>(num * PAGE_SIZE).fill(0x00) };
}

pub fn init(prot: &LoaderArg) {
    // TODO: this order is insane, but pmm relies on vmm::layout being initialized
    vmm::init(prot);
    pmm::init(prot);

    paging::kernel_page_table::init(prot);
    allocators::slab::init_kernel_slabs();
}
