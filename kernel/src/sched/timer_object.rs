use super::timer::{set_timer, TimerHandle};
use crate::object::KernelObjectBase;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::time::Duration;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;
use rtl::signal::Signal;

pub struct TimerObject {
    base: KernelObjectBase,
    handle: Spinlock<Option<TimerHandle>>,
}

crate::kernel_object!(TimerObject, Signal::TimerReady.into());

impl TimerObject {
    pub fn new() -> Result<Arc<Self>, ErrorType> {
        let res = Arc::try_new(Self {
            base: KernelObjectBase::new(),
            handle: Spinlock::new(None),
        })
        .map_err(|_| ErrorType::NoMemory)?;

        Ok(res)
    }

    pub fn arm(self: Arc<Self>, dl: Duration) -> Result<(), ErrorType> {
        let clone = self.clone();
        let mut handle = self.handle.lock();

        if let Some(handle) = handle.take() {
            handle.cancel();
        }

        // Here we are sure that no singal will arrive anymore, since cancel was successful.
        self.base.signal_clear(Signal::TimerReady.into());
        assert!(handle.is_none());

        *handle = Some(set_timer(
            dl,
            Box::try_new(move || {
                clone.signal_fire(Signal::TimerReady.into());
            })
            .map_err(|_| ErrorType::NoMemory)?,
        ));

        // loop {}
        Ok(())
    }
}
