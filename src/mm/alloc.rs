use core::alloc::{GlobalAlloc, Layout};
use crate::mm::boot_alloc;

struct Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator{};

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // if !kernel_up
        boot_alloc::alloc(layout)
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
