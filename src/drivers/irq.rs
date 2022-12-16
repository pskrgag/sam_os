use crate::kernel::sched;
use crate::{drivers::gic::GIC, kernel::locking::spinlock::Spinlock, lib::collections::list::List};

pub struct IrqHandler {
    num: u32,
    dispatcher: fn(u32),
}

pub static IRQS: Spinlock<List<IrqHandler>> = Spinlock::new(List::new());

impl IrqHandler {
    pub fn new(num: u32, func: fn(u32)) -> Self {
        Self {
            num: num,
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

    GIC.lock().enable_irq(irq);
    IRQS.lock().push(handler);
}

pub fn irq_dispatch() {
    let mut gic = GIC.lock();
    let irqs = IRQS.lock();

    for i in irqs.iter() {
        if gic.is_pending(i.num()) {
            i.dispatch();
            gic.clear_interrupt(i.num());
        }
    }
}
