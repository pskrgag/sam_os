use crate::sync::Spinlock;
use alloc::collections::LinkedList;
use arm_gic::IntId;
use rtl::irq::IrqTrigger;
use spin::Once;

pub mod gic;

pub struct IrqHandler {
    num: IntId,
    dispatcher: fn(IntId),
}

pub static IRQS: Spinlock<LinkedList<IrqHandler>> = Spinlock::new(LinkedList::new());
pub static CONTROLLER: Once<&'static dyn IrqController> = Once::new();

pub(crate) fn register_controller(controller: &'static dyn IrqController) {
    assert!(CONTROLLER.poll().is_none());

    CONTROLLER.call_once(|| controller);
}

impl IrqHandler {
    pub fn new(num: IntId, func: fn(IntId)) -> Self {
        Self {
            num,
            dispatcher: func,
        }
    }

    pub fn num(&self) -> IntId {
        self.num
    }
}

pub fn register_handler(irq: IntId, func: fn(IntId), trigger: IrqTrigger) {
    let handler = IrqHandler::new(irq, func);

    IRQS.lock().push_back(handler);
    CONTROLLER.get().unwrap().enable_irq(irq, trigger);
}

pub fn irq_dispatch() {
    let controller = CONTROLLER.get().unwrap();

    if let Some(pending) = controller.pending()
        && let Some(x) = IRQS.lock().iter().find(|x| x.num() == pending)
    {
        (x.dispatcher)(pending);
        controller.eoi(pending);
    }
}

pub trait IrqController: Send + Sync {
    fn enable_irq(&self, num: IntId, trigger: IrqTrigger);
    fn pending(&self) -> Option<IntId>;
    fn eoi(&self, int: IntId);
}
