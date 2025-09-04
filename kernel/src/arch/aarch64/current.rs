use crate::kernel::object::thread_object::Thread;
use alloc::sync::Arc;
use core::arch::asm;

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

pub fn get_current_raw() -> Option<*mut Thread> {
    unsafe {
        let raw: *mut Thread;
        asm!("mrs   {}, TPIDR_EL1", out(reg) raw);

        if raw.is_null() { None } else { Some(raw) }
    }
}

pub fn set_current(cur: Arc<Thread>) {
    unsafe {
        let raw = Arc::into_raw(cur.clone());

        asm!("msr   TPIDR_EL1, {}", in(reg) raw);
    }
}
