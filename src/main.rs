#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_trait_impl)]
#![feature(default_alloc_error_handler)]
#![feature(generic_const_exprs)]
#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(special_module_name)]

extern crate alloc;

mod drivers;
#[macro_use]
mod lib;
mod arch;
mod kernel;
mod mm;
mod panic;

#[cfg(test)]
#[macro_use]
extern crate std;

use core::arch::asm;

pub use lib::printf::*;

extern "C" {
    static __STACK_START: usize;
    fn map();
}

#[no_mangle]
extern "C" fn start_kernel() -> ! {
    println!("Starting kernel...\n");
   // arch::mm::mmu::init();
    arch::interrupts::set_up_vbar();

    mm::boot_alloc::init();
    mm::page_alloc::init();

    loop {}
}
