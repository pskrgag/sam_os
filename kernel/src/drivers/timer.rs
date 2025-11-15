use crate::arch::timer::{SYSTEM_TIMER, TIMER_IRQ_NUM};
use crate::drivers::irq::irq;
use crate::kernel::sched::current;

pub trait SystemTimer {
    fn enable(&self);
    fn reprogram(&self);
}

pub fn init() {
    SYSTEM_TIMER.reprogram();
    SYSTEM_TIMER.enable();

    irq::register_handler(TIMER_IRQ_NUM, timer_dispatch);
}

pub fn init_secondary() {
    // SYSTEM_TIMER.reprogram();
    // SYSTEM_TIMER.enable();
    //
    // // irq::init_secondary(30);
}

pub fn reprogram() {
    SYSTEM_TIMER.reprogram();
}

pub fn timer_dispatch(_: u32) {
    if let Some(cur) = current() {
        cur.tick();
    }

    reprogram();
}
