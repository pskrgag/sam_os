use crate::{
    arch::PAGE_SIZE,
    mm::{
        allocators::page_alloc::page_allocator,
        paging::{kernel_page_table::kernel_page_table, page_table::*},
        types::*,
    },
    percpu_global,
};

use spin::Once;

const KERNEL_STACK_PAGES: usize = 2;

percpu_global!(
    pub static KERNEL_STACKS: Once<VirtAddr> = Once::new();
);

pub fn init_kernel_stacks() {
    let mut stack = VirtAddr::from(
        page_allocator()
            .alloc((KERNEL_STACK_PAGES + 2) * 2)
            .expect("Failed to allocate kernel stacks"),
    );

    for i in 0..2 {
        stack.add(KERNEL_STACK_PAGES + PAGE_SIZE);

        unsafe {
            KERNEL_STACKS.cpu(i).call_once(|| stack);
        }

        stack.add(PAGE_SIZE);
    }
}
