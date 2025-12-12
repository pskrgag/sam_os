use crate::arch::backtrace::backtrace;
use core::panic::PanicInfo;
use heapless::String;
use hal::address::*;

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
        println!("Kernel Panic! In context of '{}'", id.unwrap());

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
