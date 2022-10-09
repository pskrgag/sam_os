use core::panic::PanicInfo;

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    println!("Kernel Panic!\n");
    
    if let Some(location) = info.location() {
        println!("Happened in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    }

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("Reason: {s:?}");
    }
    
    loop {}
}

