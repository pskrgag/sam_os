use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use rtl::error::ErrorType;
use rtl::irq::IrqTrigger;
use spin::Once;

pub mod gic;

pub type IntId = arm_gic::IntId;

pub struct IrqHandler {
    num: IntId,
    dispatcher: Box<dyn Fn(IntId) + Send>,
}

pub static IRQS: Spinlock<LinkedList<IrqHandler>> = Spinlock::new(LinkedList::new());
pub static CONTROLLER: Once<&'static dyn IrqController> = Once::new();

pub(crate) fn register_controller(controller: &'static dyn IrqController) {
    assert!(CONTROLLER.poll().is_none());

    CONTROLLER.call_once(|| controller);
}

impl IrqHandler {
    pub fn new<F: Fn(IntId) + Send + 'static>(num: IntId, func: F) -> Result<Self, ErrorType> {
        Ok(Self {
            num,
            dispatcher: Box::try_new(func).map_err(|_| ErrorType::NoMemory)?,
        })
    }

    pub fn num(&self) -> IntId {
        self.num
    }
}

pub fn register_handler<F: Fn(IntId) + Send + 'static>(
    irq: IntId,
    func: F,
    trigger: IrqTrigger,
) -> Result<(), ErrorType> {
    let handler = IrqHandler::new(irq, func)?;
    let mut handlers = IRQS.lock();

    if handlers.iter().find(|x| x.num == irq).is_some() {
        return Err(ErrorType::AlreadyExists);
    }

    handlers.push_back(handler);
    CONTROLLER.get().unwrap().enable_irq(irq, trigger);
    Ok(())
}

// TODO: better to return a struct from register_handler and make these function part of this struct
pub fn mask(irq: IntId) {
    CONTROLLER.get().unwrap().mask_irq(irq, true);
}

pub fn unmask(irq: IntId) {
    CONTROLLER.get().unwrap().mask_irq(irq, false);
}

pub fn unregister_handler(irq: IntId) -> Result<(), ErrorType> {
    CONTROLLER.get().unwrap().mask_irq(irq, true);
    IRQS.lock().extract_if(|x| x.num == irq).next().unwrap();
    Ok(())
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
    fn mask_irq(&self, num: IntId, mask: bool);
    fn pending(&self) -> Option<IntId>;
    fn eoi(&self, int: IntId);
}
