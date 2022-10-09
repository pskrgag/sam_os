// FIXME one day...
#[path = "../arch/aarch64/qemu/config.rs"]
mod config;

use core::{ptr, fmt};

pub struct Uart;

/* ToDo add time stamp */
pub fn uart_write(str: &[u8]) {
    for i in str {
        unsafe {
            ptr::write_volatile(config::uart_base(), *i);
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart_write(s.as_bytes());
        Ok(())
    }
}

pub fn uart() -> Uart {
    Uart { }
}

