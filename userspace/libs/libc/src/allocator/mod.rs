use crate::vmm::vms::vms;
use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
use dlmalloc::{Allocator as AllocatorApi, Dlmalloc};
use rtl::locking::spinlock::Spinlock;
use rtl::vmm::MappingType;

struct Allocator(Spinlock<Dlmalloc<PageAllocator>>);

#[cfg(not(target_os = "linux"))]
#[global_allocator]
static ALLOCATOR: Allocator = Allocator(Spinlock::new(Dlmalloc::new_with_allocator(PageAllocator)));

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().malloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().free(ptr, layout.size(), layout.align());
    }
}

struct PageAllocator;

unsafe impl AllocatorApi for PageAllocator {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        let size = if size == 0 {
            self.page_size()
        } else {
            size.next_multiple_of(self.page_size())
        };

        let ptr = vms()
            .vm_allocate(size, MappingType::Data)
            .unwrap_or(core::ptr::null_mut());

        assert!(!ptr.is_null());
        (ptr, size, 0)
    }

    fn remap(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        // TODO: why?
        core::ptr::null_mut()
    }

    fn free_part(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        // TODO: investigate
        false
    }

    fn free(&self, _ptr: *mut u8, _size: usize) -> bool {
        // TODO: implement
        false
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        false
    }

    fn allocates_zeros(&self) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        hal::arch::PAGE_SIZE
    }
}
