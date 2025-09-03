pub mod allocators;
pub mod paging;
pub mod sections;
pub mod user_buffer;
pub mod vma_list;
pub mod vms;
pub mod layout;

pub fn init() {
    allocators::boot_alloc::init();
    allocators::page_alloc::init();
    paging::kernel_page_table::init();
    sections::remap_kernel();
    paging::init_linear_map();
    allocators::slab::init_kernel_slabs();
}
