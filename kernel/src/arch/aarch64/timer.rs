use crate::drivers::timer::SystemTimer;
use arm_gic::IntId;
use aarch64_cpu::registers::{CNTFRQ_EL0, Readable, CNTP_TVAL_EL0, Writeable, CNTP_CTL_EL0};

pub struct ArmSystemTimer;

pub const TIMER_IRQ_NUM: IntId = IntId::ppi(14);
pub static SYSTEM_TIMER: ArmSystemTimer = ArmSystemTimer;

impl SystemTimer for ArmSystemTimer {
    fn enable(&self) {
        CNTP_CTL_EL0.set(1);
    }

    fn reprogram(&self) {
        let cur_freq: u64 = CNTFRQ_EL0.get();

        CNTP_TVAL_EL0.set(cur_freq / 100);
    }
}
