use crate::drivers::irq;
use crate::kernel::sched::current;
use crate::kernel::threading::thread::ThreadState;
use core::arch::asm;

pub fn init(msec: usize) {
    reprogram(msec);

    unsafe {
        asm!("mov x0, #1");
        asm!("msr CNTP_CTL_EL0, x0");
    }

    irq::register_handler(30, timer_dispatch);
}

pub fn init_secondary(msec: usize) {
    reprogram(msec);

    unsafe {
        asm!("mov x0, #1");
        asm!("msr CNTP_CTL_EL0, x0");
    }

    irq::init_secondary(30);
}

pub fn reprogram(msec: usize) {
    let mut cur_freq: usize;

    unsafe {
        asm!("mrs {}, CNTFRQ_EL0", out(reg) cur_freq);

        cur_freq += msec / 100;

        asm!("msr CNTP_TVAL_EL0, {}", in(reg) cur_freq);
    }
}

fn timer_dispatch(_: u32) {
    unsafe {
        asm!("mrs x0, cntfrq_el0");
        asm!("msr cntp_tval_el0, x0");
    }

    if let Some(cur) = current() {
        cur.write().set_state(ThreadState::NeedResched);
    }
}
