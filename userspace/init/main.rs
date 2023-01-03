#![no_std]
#![no_main]
#![feature(start)]

use core::panic::PanicInfo;

use libc::syscalls::write;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("Hello from userspace");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
