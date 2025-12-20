use crate::arch::backtrace::backtrace;
use core::panic::PanicInfo;
use hal::address::*;
use heapless::String;

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    let mut bt = [VirtAddr::from(0); 50];

    unsafe {
        let fp: usize;

        arm_gic::irq_disable();
        let id: Result<String<100>, _> = crate::sched::current().task().name().try_into();
        error!("--- cut here ---\n");
        error!("Kernel Panic! In context of '{}'\n", id.unwrap());

        error!("{}", info.message());

        if let Some(location) = info.location() {
            error!(
                "Happened in file '{}' at line {}\n",
                location.file(),
                location.line(),
            );
        }
        core::arch::asm!("mov {}, fp", out(reg) fp);

        backtrace(&mut bt, VirtAddr::from(fp));
    };

    error!("Kernel backtrace\n");
    for (i, addr) in bt.iter().take_while(|x| !x.is_null()).enumerate() {
        error!("#{} [{:p}]\n", i, addr.to_raw::<usize>());
    }

    loop {}
}
