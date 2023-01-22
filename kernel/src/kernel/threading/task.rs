use crate::{
    kernel::locking::spinlock::Spinlock, kernel::object::handle_table::HandleTable,
    mm::allocators::slab::SlabAllocator, mm::vms::Vms,
};
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Once;

const MAX_TASK_NAME: usize = 256;

struct Task {
    vms: Arc<Vms>,
    table: HandleTable,
    name: [u8; MAX_TASK_NAME],
}

static TASK_SLAB: Once<Spinlock<SlabAllocator>> = Once::new();

pub struct TaskAlloc;

// Any idea how to remove this bolerpalate? Macros? TF Rust?
unsafe impl core::alloc::Allocator for TaskAlloc {
    fn allocate(
        &self,
        _layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        let res = unsafe { TASK_SLAB.get_unchecked().lock().alloc().unwrap() };

        Ok(
            unsafe {
                core::ptr::NonNull::new(
                    core::slice::from_raw_parts_mut(res as *mut u8, core::mem::size_of::<Task>())
                    )
                    .unwrap()
            }
        )
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        TASK_SLAB.get_unchecked().lock().free(ptr.as_ptr());
    }
}

impl Task {
    fn new(name: &[u8]) -> Option<Box<Self, TaskAlloc>> {
        TASK_SLAB.call_once(|| Spinlock::new(SlabAllocator::new(
            core::mem::size_of::<Task>(),
        ).unwrap()));

        Some(Box::<Self, TaskAlloc>::new_in(
            Self {
                vms: Arc::new(Vms::empty()?),
                table: HandleTable::new(),
                name: name.try_into().ok()?,
            }, TaskAlloc{},
        ))
    }
}
