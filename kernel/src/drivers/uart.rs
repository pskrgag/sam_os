use crate::drivers::mmio_mapper::MMIO_ALLOCATOR;
use crate::kernel::locking::spinlock::Spinlock;
use crate::mm::types::*;
use core::{fmt, ptr};

pub struct Uart;

static UART_PTR: Spinlock<VirtAddr> = Spinlock::new(VirtAddr::new(0x09000000));

pub fn uart_write(str: &[u8]) {
    let ptr = UART_PTR.lock_irqsave();

    for i in str {
        unsafe {
            ptr::write_volatile(ptr.to_raw_mut::<u8>(), *i);
        }

        if *i == b'\n' {
            unsafe {
                ptr::write_volatile(ptr.to_raw_mut::<u8>(), b'\r');
            }
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart_write(s.as_bytes());
        Ok(())
    }
}

pub fn remap() {
    let mut p = UART_PTR.lock();
    let alloc = MMIO_ALLOCATOR.get();
    let ptr = alloc
        .iomap(PhysAddr::new(p.bits()), 1)
        .expect("Failed to remap uart");

    *p = ptr;
}

pub fn uart() -> Uart {
    Uart {}
}
