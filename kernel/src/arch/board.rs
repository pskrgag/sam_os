pub trait Board {
    fn ram_base() -> *mut u8;
    fn ram_size() -> usize;
    fn uart_base() -> *mut u8;
}
