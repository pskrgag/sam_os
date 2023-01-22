use crate::{
    arch::PAGE_SIZE,
    mm::{
        allocators::page_alloc::page_allocator,
        paging::{kernel_page_table::kernel_page_table, page_table::*},
        types::*,
    },
};

use spin::Once;

const KERNEL_STACK_PAGES: usize = 2;

// TODO: NUM_CPUS
pub static KERNEL_STACKS: Once<[VirtAddr; 2]> = Once::new();

pub fn init_kernel_stacks() {
    let mut stack = VirtAddr::from(PhysAddr::from(
        page_allocator().alloc((KERNEL_STACK_PAGES + 2) * 2).expect("Failed to allocate kernel stacks"),
    ));
    let mut kernel_stacks: [VirtAddr; 2] = [VirtAddr::from(0); 2];

    for i in 0..2 {
        kernel_stacks[i] = *stack.add(KERNEL_STACK_PAGES + PAGE_SIZE);
        stack.add(PAGE_SIZE);
    }

    KERNEL_STACKS.call_once(|| kernel_stacks);
}
