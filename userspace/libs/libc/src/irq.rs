use crate::factory::factory;
use crate::handle::Handle;
use rtl::error::ErrorType;
use rtl::irq::IrqTrigger;

pub struct Irq {
    h: Handle,
}

impl Irq {
    pub unsafe fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn create(num: usize, trigger: IrqTrigger) -> Result<Self, ErrorType> {
        factory().create_irq(num, trigger)
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }

    pub fn ack(&self) -> Result<(), ErrorType> {
        crate::syscalls::Syscall::ack_irq(&self.h)
    }

    pub fn try_clone(&self) -> Result<Self, ErrorType> {
        let res = self.h.clone_handle()?;

        unsafe { Ok(Self::new(res)) }
    }
}
