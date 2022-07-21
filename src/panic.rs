use core::panic::PanicInfo;
use crate::lib::printf;

#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    printf::printf(b"Kernel Panic\n");
    loop {}
}

