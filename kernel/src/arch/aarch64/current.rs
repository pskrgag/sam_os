use alloc::sync::Arc;
use core::arch::asm;
use qrwlock::RwLock;
use crate::kernel::tasks::thread::{Thread, ThreadRef};

pub fn get_current() -> Option<ThreadRef> {
    unsafe {
        let raw: *const RwLock<Thread>;
        asm!("mrs   {}, TPIDR_EL1", out(reg) raw);

        if raw.is_null() {
            None
        } else {
            Some(Arc::from_raw(raw))
        }
    }
}

pub fn set_current(cur: ThreadRef) {
    unsafe {
        let raw = Arc::into_raw(cur);

        asm!("msr   TPIDR_EL1, {}", in(reg) raw);
    }
}
