use core::ptr;

const UART_BASE: *mut u8 = 0x0900_0000 as *mut u8;

/* ToDo add time stamp */
pub fn uart_write(str: &[u8]) {
    for i in str {
        unsafe {
            ptr::write_volatile(UART_BASE, *i);
        }
    }
}
