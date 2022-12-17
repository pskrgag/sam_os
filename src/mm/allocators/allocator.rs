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
        if !BOOT_ALLOC_IS_DEAD.load(Ordering::Relaxed) {
            let boot_alloc = boot_alloc::BOOT_ALLOC.get();

            boot_alloc.alloc(layout)
        } else {
            slab::alloc(layout.size()).unwrap()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !BOOT_ALLOC_IS_DEAD.load(Ordering::Relaxed) {
            let boot_alloc = boot_alloc::BOOT_ALLOC.get();

            boot_alloc.free(ptr)
        } else {
            slab::free(ptr, layout);
        }
    }
}
