use crate::smp::percpu_ready;
use crate::tasks::thread::Thread;
use alloc::sync::Arc;

percpu_global!(
    static CURRENT: Option<Arc<Thread>> = None;
);

// Using lazy_static! here, since LazyCell cannot be shared between threads...
lazy_static::lazy_static! {
    static ref DUMMY: Arc<Thread> = Thread::initial().unwrap();
}

pub fn get_current() -> Arc<Thread> {
    CURRENT.per_cpu_var_get().as_ref().unwrap_or(&DUMMY).clone()
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
