use crate::sched::timer::sched_tick;
use core::sync::atomic::{AtomicU64, Ordering::Relaxed};
use core::time::Duration;

pub type SchedTicks = u64;
pub const SYSTEM_TICK: Duration = Duration::from_millis(10);

percpu_global! {
    static SCHED_TICKS: AtomicU64 = AtomicU64::new(0);
}

pub fn tick() {
    SCHED_TICKS.per_cpu_var_get().fetch_add(1, Relaxed);
    sched_tick();
}

pub fn sched_ticks() -> SchedTicks {
    SCHED_TICKS.per_cpu_var_get().load(Relaxed)
}
