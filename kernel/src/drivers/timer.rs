use crate::arch::timer::{SYSTEM_TIMER, TIMER_IRQ_NUM};
use crate::drivers::irq::{gic::ClaimedIrq, irq};
use crate::kernel::sched::current;

pub trait SystemTimer {
    fn enable(&self);
    fn reprogram(&self);
}

pub fn init() {
    irq::register_handler(TIMER_IRQ_NUM, timer_dispatch);

    SYSTEM_TIMER.reprogram();
    SYSTEM_TIMER.enable();
}

pub fn timer_dispatch(_: &ClaimedIrq) {
    if let Some(cur) = current() {
        cur.tick();
    }

    SYSTEM_TIMER.reprogram();
}
