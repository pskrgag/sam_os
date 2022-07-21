use crate::drivers::uart;

pub fn printf(str: &[u8]) {
    uart::uart_write(str)
}
