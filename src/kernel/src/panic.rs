#[cfg(not(test))]
use crate::arch::{backtrace::backtrace, irq::interrupts::disable_all};

use core::panic::PanicInfo;
use crate::alloc::borrow::ToOwned;

#[cfg(not(test))]
use rtl::vmm::types::*;

#[cfg(test)]
#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    println!("{}", info.message());

    if let Some(location) = info.location() {
        println!(
            "Happened in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    }

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    let mut bt = [VirtAddr::from(0); 50];

    unsafe {
        let fp: usize;

        disable_all();
        let id = if let Some(c) = crate::sched::current() {
            c.task().name().to_owned()
        } else {
            "kernel".to_owned()
        };
        println!("--- cut here ---");
        println!("Kernel Panic! In context of '{}'", id);

        println!("{}", info.message());

        if let Some(location) = info.location() {
            println!(
                "Happened in file '{}' at line {}",
                location.file(),
                location.line(),
            );
        }
        core::arch::asm!("mov {}, fp", out(reg) fp);

        backtrace(&mut bt, VirtAddr::from(fp));
    };

    println!("Kernel backtrace");
    for (i, addr) in bt.iter().take_while(|x| !x.is_null()).enumerate() {
        println!("#{} [{:p}]", i, addr.to_raw::<usize>());
    }

    loop {}
}
