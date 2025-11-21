use crate::drivers::timer::SystemTimer;
use arm_gic::IntId;
use core::arch::asm;

pub struct ArmSystemTimer;

pub const TIMER_IRQ_NUM: IntId = IntId::ppi(14);
pub static SYSTEM_TIMER: ArmSystemTimer = ArmSystemTimer;

impl SystemTimer for ArmSystemTimer {
    fn enable(&self) {
        unsafe {
            asm!("mov x0, #1", "msr CNTP_CTL_EL0, x0");
        }
    }

    // NOTE: rust generates weird code with -O1+ for some reason.
    // Leave it as noinline for now to w/a it
    #[inline(never)]
    fn reprogram(&self) {
        let mut cur_freq: u64;

        unsafe {
            asm!("mrs {}, CNTFRQ_EL0", out(reg) cur_freq);
            cur_freq /= 100;
            asm!("msr CNTP_TVAL_EL0, {}", in(reg) cur_freq);
        }
    }
}
