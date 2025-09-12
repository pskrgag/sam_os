pub mod allocators;
pub mod paging;
pub mod user_buffer;
pub mod vma_list;
pub mod vms;
pub mod layout;

pub fn init() {
    allocators::boot_alloc::init();
    allocators::page_alloc::init();
    paging::kernel_page_table::init();
    allocators::slab::init_kernel_slabs();
}
