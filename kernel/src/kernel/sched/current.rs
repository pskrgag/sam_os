use crate::kernel::object::thread_object::Thread;
use crate::kernel::percpu::percpu_ready;
use crate::percpu_global;
use alloc::sync::Arc;

percpu_global!(
    static CURRENT: Option<Arc<Thread>> = None;
);

pub fn get_current() -> Option<Arc<Thread>> {
    CURRENT.per_cpu_var_get().clone()
}

pub fn get_current_raw() -> Option<*const Thread> {
    if !percpu_ready() {
        None
    } else {
        CURRENT.per_cpu_var_get().clone().map(|x| Arc::as_ptr(&x))
    }
}

pub fn set_current(cur: Arc<Thread>) {
    *CURRENT.per_cpu_var_get_mut() = Some(cur);
}
