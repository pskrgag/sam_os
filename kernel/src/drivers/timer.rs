use crate::arch::timer::{SYSTEM_TIMER, TIMER_IRQ_NUM};
use crate::drivers::irq::register_handler;
use crate::sched::{current, ticks::SYSTEM_TICK};
use arm_gic::IntId;
use core::time::Duration;
use rtl::irq::IrqTrigger;

pub trait SystemTimer {
    fn enable(&self);
    fn reprogram(&self, dur: Duration);
    fn since_start(&self) -> Duration;
}

pub fn init() {
    register_handler(TIMER_IRQ_NUM, timer_dispatch, IrqTrigger::Level).unwrap();

    SYSTEM_TIMER.reprogram(SYSTEM_TICK);
    SYSTEM_TIMER.enable();
}

pub fn timer_dispatch(_: IntId) {
    current().tick();

    crate::sched::ticks::tick();
    SYSTEM_TIMER.reprogram(SYSTEM_TICK);
}
