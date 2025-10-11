use super::gic::GIC;
use crate::kernel::locking::spinlock::Spinlock;
use alloc::collections::LinkedList;

pub struct IrqHandler {
    num: u32,
    dispatcher: fn(u32),
}

pub static IRQS: Spinlock<LinkedList<IrqHandler>> = Spinlock::new(LinkedList::new());

impl IrqHandler {
    pub fn new(num: u32, func: fn(u32)) -> Self {
        Self {
            num,
            dispatcher: func,
        }
    }

    pub fn num(&self) -> u32 {
        self.num
    }

    pub fn dispatch(&self) {
        (self.dispatcher)(self.num);
    }
}

pub fn register_handler(irq: u32, func: fn(u32)) {
    let handler = IrqHandler::new(irq, func);

    GIC.get().unwrap().lock().enable_irq(irq);
    IRQS.lock().push_back(handler);
}

pub fn init_secondary(_irq: u32) {
    // use crate::arch::cpuid::current_cpu;
    //
    // GIC.per_cpu_var_get_mut()
    //     .init_secondary(irq, current_cpu() as u32);
}

pub fn irq_dispatch() {
    let mut cur_freq: u64;

    unsafe {
        core::arch::asm!("mrs {}, CNTFRQ_EL0", out(reg) cur_freq);
        cur_freq /= 50;
        core::arch::asm!("msr CNTP_TVAL_EL0, {}", in(reg) cur_freq);
    }

    // let mut gic = GIC.get().unwrap().lock();
    // let irqs = IRQS.lock();
    //
    // for i in irqs.iter() {
    //     if gic.is_pending(i.num()) {
    //         i.dispatch();
    //         gic.clear_interrupt(i.num());
    //     }
    // }
}
