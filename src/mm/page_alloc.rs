// FIXME one day...
#[path = "../arch/aarch64/qemu/config.rs"]
mod config;

use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};

static PAGE_ALLOC: NonThreadsafeAlloc = unsafe {
    let fast_param = FastAllocParam::new(0 as *mut _, 0);
    let buddy_param = BuddyAllocParam::new(config::ram_base(), config::ram_size(), 4096);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};
