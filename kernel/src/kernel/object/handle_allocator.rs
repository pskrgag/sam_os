use crate::{
    kernel::{locking::spinlock::Spinlock, object::handle::Handle},
    mm::allocators::slab::{SlabAllocator, SlabPolicy},
};
use core::mem::size_of;
use spin::Once;

pub struct HandleSlabPolicy(());

impl SlabPolicy for HandleSlabPolicy {
    const MAX_SLABS: Option<usize> = Some(1000);
    type ObjectType = Handle;
}

pub static HANDLE_ALLOC: Once<Spinlock<SlabAllocator<HandleSlabPolicy>>> = Once::new();

pub fn init() {
    HANDLE_ALLOC.call_once(|| {
        Spinlock::new(
            SlabAllocator::<HandleSlabPolicy>::new(size_of::<Handle>())
                .expect("Failed to init handle slab"),
        )
    });
}
