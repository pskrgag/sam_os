use crate::drivers::irq::irq;
use crate::kernel::sched::current;
use core::arch::asm;

pub fn init() {
    reprogram();

    unsafe {
        asm!("mov x0, #1");
        asm!("msr CNTP_CTL_EL0, x0");
        crate::arch::irq::enable_all();
    }

    irq::register_handler(30, timer_dispatch);
}

pub fn init_secondary() {
    reprogram();

    unsafe {
        asm!("mov x0, #1");
        asm!("msr CNTP_CTL_EL0, x0");
    }

    irq::init_secondary(30);
}

pub fn reprogram() {
    let mut cur_freq: usize;

    unsafe {
        asm!("mrs {}, CNTFRQ_EL0", out(reg) cur_freq);

        cur_freq /= 100;

        asm!("msr CNTP_TVAL_EL0, {}", in(reg) cur_freq);
    }
}

fn timer_dispatch(_: u32) {
    if let Some(cur) = current() {
        cur.tick();
    }

    unsafe {
        let mut t: usize;
        asm!("mrs {}, cntfrq_el0", out(reg) t);
        t /= 100;
        asm!("msr cntp_tval_el0, {}", in(reg) t);
    }
}
