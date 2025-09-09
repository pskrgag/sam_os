#![no_std]
#![no_main]
#![feature(used_with_arg)]

use core::panic::PanicInfo;
use rtl::vmm::types::{PhysAddr, Address};
use fdt::Fdt;

mod arch;
#[macro_use]
mod log;
mod drivers;

#[unsafe(no_mangle)]
extern "C" fn main(fdt: PhysAddr) {
    let fdt = unsafe { Fdt::from_ptr(fdt.bits() as *const _) }.unwrap();

    drivers::uart::probe(&fdt);

    loop {}
}

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    println!("panic! {}", info.message());
    loop {}
}
