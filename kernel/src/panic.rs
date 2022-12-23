use crate::{
    arch::{backtrace::backtrace, irq::disable_all},
    mm::types::VirtAddr,
};
use core::panic::PanicInfo;

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    let mut bt = [VirtAddr::from(0); 50];
    let bt_size: usize;

    unsafe {
        disable_all();
        println!("--- cut here ---");
        println!("Kernel Panic!");
        bt_size = backtrace(&mut bt);
    };

    if let Some(location) = info.location() {
        println!(
            "Happened in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    }

    println!("Kernel backtrace");
    for i in 0..bt_size {
        println!("#{} [{:p}]", i, bt[i].to_raw::<usize>());
    }

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("Reason: {s:?}");
    }

    loop {}
}
