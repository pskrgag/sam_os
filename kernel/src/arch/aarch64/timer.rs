use crate::drivers::timer::SystemTimer;
use aarch64_cpu::registers::{
    CNTFRQ_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0, CNTPCT_EL0, Readable, Writeable,
};
use arm_gic::IntId;
use core::time::Duration;

pub struct ArmSystemTimer;

pub const TIMER_IRQ_NUM: IntId = IntId::ppi(14);
pub static SYSTEM_TIMER: ArmSystemTimer = ArmSystemTimer;

impl SystemTimer for ArmSystemTimer {
    fn enable(&self) {
        CNTP_CTL_EL0.set(1);
    }

    fn reprogram(&self, dur: Duration) {
        let cur_freq: u64 = CNTFRQ_EL0.get();
        let ms = dur.as_millis() as u64;
        // Hz is number of ticks per second. There are 1000 ms in second, so hz / 1000 is number of
        // ticks per ms
        let ticks_per_ms = cur_freq / 1000;

        CNTP_TVAL_EL0.set(ticks_per_ms * ms);
    }

    fn since_start(&self) -> Duration {
        let cntfrq = CNTFRQ_EL0.get();
        let cntpct = CNTPCT_EL0.get() * 1_000_000_000;

        Duration::from_nanos(cntpct / cntfrq)
    }
}
