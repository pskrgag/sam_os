use alloc::sync::Arc;
use core::arch::asm;
use crate::kernel::object::thread_object::Thread;

pub fn get_current() -> Option<Arc<Thread>> {
    unsafe {
        let raw: *const Thread;
        asm!("mrs   {}, TPIDR_EL1", out(reg) raw);

        if raw.is_null() {
            None
        } else {
            Arc::increment_strong_count(raw);
            Some(Arc::from_raw(raw))
        }
    }
}

pub fn set_current(cur: Arc<Thread>) {
    unsafe {
        let raw = Arc::into_raw(cur.clone());

        asm!("msr   TPIDR_EL1, {}", in(reg) raw);
    }
}
