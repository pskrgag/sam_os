use crate::kernel::locking::spinlock::Spinlock;
use core::fmt;
use rtl::uart::Uart as BackendUart;
use rtl::uart::UartTrait;
use hal::address::*;

pub struct Uart;

pub static UART: Spinlock<BackendUart> = Spinlock::new(BackendUart::invalid());

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        UART.lock_irqsave().write_bytes(s.as_bytes());
        Ok(())
    }
}

pub fn remap(base: VirtAddr) {
    let mut p = UART.lock();
    *p = BackendUart::new(base);
}

pub fn uart() -> Uart {
    Uart {}
}
