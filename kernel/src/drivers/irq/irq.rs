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

    GIC.get().enable_irq(irq);
    IRQS.lock().push_back(handler);
}

pub fn init_secondary(irq: u32) {
    use crate::arch::cpuid::current_cpu;

    GIC.get().init_secondary(irq, current_cpu() as u32);
}

pub fn irq_dispatch() {
    let gic = GIC.get();
    let irqs = IRQS.lock();

    for i in irqs.iter() {
        if gic.is_pending(i.num()) {
            i.dispatch();
            gic.clear_interrupt(i.num());
        }
    }
}
