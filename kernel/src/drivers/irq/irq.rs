use super::gic::{ClaimedIrq, GIC};
use crate::sync::Spinlock;
use alloc::collections::LinkedList;
use arm_gic::IntId;

pub struct IrqHandler {
    num: IntId,
    dispatcher: fn(&ClaimedIrq),
}

pub static IRQS: Spinlock<LinkedList<IrqHandler>> = Spinlock::new(LinkedList::new());

impl IrqHandler {
    pub fn new(num: IntId, func: fn(&ClaimedIrq)) -> Self {
        Self {
            num,
            dispatcher: func,
        }
    }

    pub fn num(&self) -> IntId {
        self.num
    }
}

pub fn register_handler(irq: IntId, func: fn(&ClaimedIrq)) {
    let handler = IrqHandler::new(irq, func);

    GIC.get().unwrap().lock().enable_irq(irq);
    IRQS.lock().push_back(handler);
}

// pub fn init_secondary(_irq: u32) {
// use crate::arch::cpuid::current_cpu;
//
// GIC.per_cpu_var_get_mut()
//     .init_secondary(irq, current_cpu() as u32);
// }

pub fn irq_dispatch() {
    let gic = GIC.get().unwrap().lock();

    gic.pending().map(|pending| {
        if let Some(x) = IRQS.lock().iter().find(|x| x.num() == pending.0) {
            (x.dispatcher)(&pending);
        }
    });
}
