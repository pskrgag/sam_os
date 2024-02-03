use crate::arch::uart_base;
use crate::drivers::mmio_mapper::MMIO_ALLOCATOR;
use crate::kernel::locking::spinlock::Spinlock;
use core::fmt;
use rtl::uart::Uart as BackendUart;
use rtl::uart::UartTrait;
use rtl::vmm::types::*;

pub struct Uart;

static UART: Spinlock<BackendUart> =
    Spinlock::new(BackendUart::default(VirtAddr::new(0x01C2_8000)));

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
