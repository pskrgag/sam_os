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
mod panic;
mod arch;
mod kernel;
mod mm;

#[cfg(test)]
#[macro_use]
extern crate std;

use core::arch::asm;
use mm::{
    types::*,
    boot_alloc::BOOT_ALLOC,
    page_alloc::{self, PAGE_ALLOC},
};

pub use lib::printf::*;

extern "C" {
    static __STACK_START: usize;
}

#[no_mangle]
fn start_kernel() -> ! {
    println!("Starting kernel....\n");
    arch::interrupts::set_up_vbar();
    arch::mm::mmu::init();

    mm::boot_alloc::init();
    mm::page_alloc::init();

    loop {  }
}

#[no_mangle]
#[link_section = ".text.boot"]
pub unsafe extern "C" fn _start() -> ! {
    unsafe {
         asm!("mov sp, {}", in(reg) &__STACK_START);
    }

     start_kernel()
}
