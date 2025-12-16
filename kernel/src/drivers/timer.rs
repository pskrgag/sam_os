use crate::arch::timer::{SYSTEM_TIMER, TIMER_IRQ_NUM};
use crate::drivers::irq::{gic::ClaimedIrq, irq};
use crate::kernel::sched::{current, ticks::SYSTEM_TICK};
use core::time::Duration;

pub trait SystemTimer {
    fn enable(&self);
    fn reprogram(&self, dur: Duration);
    fn since_start(&self) -> Duration;
}

pub fn init() {
    irq::register_handler(TIMER_IRQ_NUM, timer_dispatch);

    SYSTEM_TIMER.reprogram(SYSTEM_TICK);
    SYSTEM_TIMER.enable();
}

pub fn timer_dispatch(_: &ClaimedIrq) {
    if let Some(cur) = current() {
        cur.tick();
    }

    crate::kernel::sched::ticks::tick();
    SYSTEM_TIMER.reprogram(SYSTEM_TICK);
}
