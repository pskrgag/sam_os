#[cfg(not(test))]
use crate::arch::{backtrace::backtrace, irq::interrupts::disable_all};

use core::panic::PanicInfo;

#[cfg(not(test))]
use rtl::vmm::types::*;

#[cfg(test)]
#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    if let Some(m) = info.message() {
        println!("{}", m);
    }
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
    let bt_size: usize;

    unsafe {
        let fp: usize;

        disable_all();
        let id = if let Some(c) = crate::sched::current() {
            c.id()
        } else {
            u16::MAX
        };
        println!("--- cut here ---");
        println!("Kernel Panic! In context of {}", id);

        if let Some(m) = info.message() {
            println!("{}", m);
        }

        if let Some(location) = info.location() {
            println!(
                "Happened in file '{}' at line {}",
                location.file(),
                location.line(),
            );
        }
        core::arch::asm!("mov {}, fp", out(reg) fp);

        bt_size = backtrace(&mut bt, VirtAddr::from(fp));
    };

    println!("Kernel backtrace");
    for i in 0..bt_size {
        println!("#{} [{:p}]", i, bt[i].to_raw::<usize>());
    }

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("Reason: {:?}", s);
    }

    loop {}
}
