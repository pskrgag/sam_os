use crate::drivers::irq::{mask, register_handler, unmask, unregister_handler, IntId};
use crate::object::KernelObjectBase;
use crate::sync::Event;
use alloc::sync::Arc;
use rtl::error::ErrorType;
use rtl::irq::IrqTrigger;
use crate::sync::Spinlock;
use rtl::signal::Signal;

enum State {
    Pending,
    Signaled,
    NeedAck,
}

struct IrqObjectInner {
    num: IntId,
    state: State,
    trigger: IrqTrigger,
}

pub struct IrqObject {
    base: KernelObjectBase,
    inner: Spinlock<IrqObjectInner>,
    event: Event,
}

crate::kernel_object!(IrqObject, Signal::IrqReady.into());

impl IrqObject {
    pub fn new(num: IntId, trigger: IrqTrigger) -> Result<Arc<Self>, ErrorType> {
        let res = Arc::try_new(Self {
            base: KernelObjectBase::new(),
            event: Event::new(),
            inner: Spinlock::new(IrqObjectInner {
                num,
                trigger,
                state: State::Pending,
            }),
        })
        .map_err(|_| ErrorType::NoMemory)?;

        let clone = res.clone();

        register_handler(
            num,
            move |_| {
                let mut inner = clone.inner.lock_irqsave();

                inner.state = State::Signaled;
                clone.base.signal_fire(Signal::IrqReady.into());
                mask(inner.num);
            },
            trigger,
        )?;

        Ok(res)
    }

    pub fn ack(&self) -> Result<(), ErrorType> {
        let mut inner = self.inner.lock_irqsave();

        match inner.state {
            State::Pending => Err(ErrorType::WouldBlock),
            State::NeedAck => {
                inner.state = State::Pending;

                self.base.signal_clear(Signal::IrqReady.into());
                unmask(inner.num);
                Ok(())
            }
            State::Signaled => {
                inner.state = State::NeedAck;
                Ok(())
            }
        }
    }
}

impl Drop for IrqObject {
    fn drop(&mut self) {
        unregister_handler(self.inner.lock().num).unwrap();
    }
}
