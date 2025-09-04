use crate::mm::allocators::{boot_alloc, slab};
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Allocator;

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator {};

pub static BOOT_ALLOC_IS_DEAD: AtomicBool = AtomicBool::new(false);

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            if !BOOT_ALLOC_IS_DEAD.load(Ordering::Relaxed) {
                boot_alloc::BOOT_ALLOC.lock().alloc(layout)
            } else {
                slab::alloc(layout.size()).unwrap()
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            if !BOOT_ALLOC_IS_DEAD.load(Ordering::Relaxed) {
                boot_alloc::BOOT_ALLOC.lock().free(ptr)
            } else {
                slab::free(ptr, layout);
            }
        }
    }
}
