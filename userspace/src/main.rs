#![no_std]
#![no_main]
#![feature(start)]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start(_a: isize, _b: *const *const u8) -> isize {
    loop {}

    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
