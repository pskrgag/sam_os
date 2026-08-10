use crate::drivers::irq::{IntId, mask, register_handler, unmask, unregister_handler};
use crate::object::KernelObjectBase;
use crate::sync::Event;
use alloc::sync::Arc;
use rtl::error::ErrorType;
use rtl::irq::IrqTrigger;
use rtl::locking::spinlock::Spinlock;
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

crate::kernel_object!(IrqObject, Signal::None.into());

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
                clone.event.broadcast();

                mask(inner.num);
            },
            trigger,
        )?;

        Ok(res)
    }

    pub async fn wait(&self) -> Result<(), ErrorType> {
        loop {
            let need_wait = {
                let mut inner = self.inner.lock_irqsave();

                match inner.state {
                    State::Pending => true,
                    State::NeedAck => {
                        inner.state = State::Pending;
                        unmask(inner.num);
                        true
                    }
                    State::Signaled => {
                        inner.state = State::NeedAck;
                        self.event.clear();
                        false
                    }
                }
            };

            if need_wait {
                self.event.wait().await?;
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl Drop for IrqObject {
    fn drop(&mut self) {
        unregister_handler(self.inner.lock().num).unwrap();
    }
}
