use crate::drivers::mmio_mapper::MMIO_ALLOCATOR;
use crate::kernel::locking::spinlock::Spinlock;
use core::fmt;
use rtl::uart::Uart as BackendUart;
use rtl::uart::UartTrait;
use rtl::vmm::types::*;
use crate::arch::uart_base;

pub struct Uart;

static UART: Spinlock<BackendUart> = Spinlock::new(BackendUart::default(uart_base()));

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        UART.lock_irqsave().write_bytes(s.as_bytes());
        Ok(())
    }
}

pub fn remap() {
    let mut p = UART.lock();
    let alloc = MMIO_ALLOCATOR.get();
    let ptr = alloc
        .iomap(PhysAddr::new(p.base().bits()), 1)
        .expect("Failed to remap uart");

    *p = BackendUart::default(ptr);
}

pub fn uart() -> Uart {
    Uart {}
}
