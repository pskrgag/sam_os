#![no_std]
#![no_main]
#![feature(start)]

use core::panic::PanicInfo;

use libc::syscalls::write;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {
        write("Hello from userspace");

        for _ in 0..10_000_000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
