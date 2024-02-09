use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;

pub mod backend;
pub mod slab;

pub struct Allocator;

#[cfg(not(target_os = "linux"))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator {};

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        slab::alloc(layout.size()).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        slab::free(ptr, layout)
    }
}
