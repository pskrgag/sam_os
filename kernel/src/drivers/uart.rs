use crate::drivers::mmio_mapper::MMIO_ALLOCATOR;
use crate::mm::types::*;
use core::{fmt, ptr};

pub struct Uart;

static mut UART_PTR: *mut u8 = 0x09000000 as *mut u8;

/* ToDo add time stamp */
pub fn uart_write(str: &[u8]) {
    for i in str {
        unsafe {
            ptr::write_volatile(UART_PTR, *i);
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
    let alloc = MMIO_ALLOCATOR.get();
    let ptr = alloc
        .iomap(PhysAddr::new(unsafe { UART_PTR as usize }), 1)
        .expect("Failed to remap uart");

    unsafe {
        UART_PTR = ptr.to_raw_mut::<u8>();
    };
}

pub fn uart() -> Uart {
    Uart {}
}
