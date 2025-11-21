#[cfg(not(test))]
use crate::arch::backtrace::backtrace;

use core::panic::PanicInfo;
use heapless::String;

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

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    let mut bt = [VirtAddr::from(0); 50];

    println!("panic");
    unsafe {
        let fp: usize;

        arm_gic::irq_disable();
        let id: Result<String<100>, _> = if let Some(c) = crate::sched::current() {
            c.task().name().try_into()
        } else {
            "kernel".try_into()
        };
        println!("--- cut here ---");
        println!("Kernel Panic! In context of '{:?}'", id);

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
