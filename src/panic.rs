use crate::arch::irq::disable_all;
use core::panic::PanicInfo;

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    unsafe { disable_all() };

    println!("--- cut here ---");
    println!("Kernel Panic!");

    if let Some(location) = info.location() {
        println!(
            "Happened in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    }

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("Reason: {s:?}");
    }

    loop {}
}
