#![no_std]
#![no_main]
#![feature(used_with_arg)]

use core::panic::PanicInfo;
use fdt::Fdt;
use hal::address::{Address, PhysAddr};
use loader_protocol::LoaderArg;

mod arch;
#[macro_use]
mod log;
mod drivers;
mod kernel;
mod mm;
mod protocol;

#[unsafe(no_mangle)]
extern "C" fn main(fdt_base: PhysAddr) {
    let mut protocol = LoaderArg::default();
    let fdt = unsafe { Fdt::from_ptr(fdt_base.bits() as *const _) }.unwrap();

    drivers::uart::probe(&fdt);

    let mut tt = mm::init(&fdt, fdt_base);
    kernel::map_kernel(&mut tt);

    mm::layout::init_layout(&mut protocol);
    mm::linear_map::map_linear(&mut tt, &protocol);

    drivers::map(&fdt, &mut protocol);
    let arg0 = protocol::prepare(fdt_base, &fdt, protocol, &mut tt);

    arch::boot::boot(kernel::kernel_ep().bits(), arg0.bits(), tt.base().bits());

    panic!("Unexpected return from kernel boot...");
}

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    println!("panic! {}", info.message());

    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
