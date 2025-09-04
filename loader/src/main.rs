#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rtl::uart::arm_uart;

mod arch;
mod log;

#[unsafe(no_mangle)]
extern "C" fn main() {
}

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    loop {}
}
