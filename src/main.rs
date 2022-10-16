#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_trait_impl)]

mod drivers;
#[macro_use]
mod lib;
mod panic;
mod arch;
mod mm;

use core::arch::asm;

pub use lib::printf::*;

extern "C" {
    static __STACK_START: usize;
}

#[no_mangle]
fn start_kernel() -> ! {
    println!("Starting kernel....\n");
    arch::interrupts::set_up_vbar();
    arch::mm::mmu::init();

    loop { }
}

#[no_mangle]
#[link_section = ".text.boot"]
pub unsafe extern "C" fn _start() -> ! {
    unsafe {
         asm!("mov sp, {}", in(reg) &__STACK_START);
    }

     start_kernel()
}
