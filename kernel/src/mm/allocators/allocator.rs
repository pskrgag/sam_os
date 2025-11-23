use crate::mm::allocators::slab;
use core::alloc::{GlobalAlloc, Layout};

pub struct Allocator;

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator {};

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        slab::alloc(layout.size()).unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        slab::free(ptr, layout);
    }
}
