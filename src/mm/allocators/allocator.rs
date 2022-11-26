use crate::mm::allocators::boot_alloc;
use core::alloc::{GlobalAlloc, Layout};

pub struct Allocator;

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator {};

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let boot_alloc = boot_alloc::BOOT_ALLOC.get();

        boot_alloc.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let boot_alloc = boot_alloc::BOOT_ALLOC.get();

        boot_alloc.free(ptr);
    }
}
