#![no_std]
#![no_main]

mod panic;
mod lib;
mod drivers;
mod arch;
mod mm;

use core::arch::global_asm;
use crate::lib::printf;
use crate::mm::page_alloc;

global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn start_kernel() {
    printf::printf(b"Main called\n");
    page_alloc::mm_set_up_memory_layout(&arch::qemu::config::MemoryLayout);
    loop {}
}

