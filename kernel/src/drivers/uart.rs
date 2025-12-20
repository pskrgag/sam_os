use crate::sync::Spinlock;
use core::fmt;
use hal::address::*;
use hal::uart::{Uart as BackendUart, UartTrait};
use spin::Once;

pub struct Uart;

pub static UART: Once<Spinlock<BackendUart>> = Once::new();

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            UART.get_unchecked()
                .lock_irqsave()
                .write_bytes(s.as_bytes())
        };
        Ok(())
    }
}

pub fn remap(base: VirtAddr) {
    UART.call_once(|| Spinlock::new(BackendUart::new(base)));
}

pub fn uart() -> Uart {
    Uart {}
}
