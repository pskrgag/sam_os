#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_trait_impl)]
#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(special_module_name)]
#![feature(int_roundings)]
#![feature(const_mut_refs)]
#![feature(linked_list_cursors)]
#![feature(allocator_api)]
#![feature(naked_functions)]

extern crate alloc;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod lib;

mod arch;
#[macro_use]
mod kernel;
mod drivers;
mod mm;
mod panic;

use arch::smp::bring_up_cpus;
use kernel::sched;
use kernel::threading::thread_ep::idle_thread;
use kernel::threading::thread_table;
pub use lib::printf;

/* At this point we have:
 *
 *      1) MMU is turned on
 *      2) MMMIO is mapped as 1 to 1
 *      3) 0xffffffffc0000000 and load_addr are mapped to load_addr via 1GB block
 */
#[no_mangle]
extern "C" fn start_kernel() -> ! {
    println!("Starting kernel...");
    arch::interrupts::set_up_vbar();

    mm::allocators::boot_alloc::init();
    mm::allocators::page_alloc::init();

    mm::paging::kernel_page_table::init();
    mm::sections::remap_kernel();

    mm::allocators::slab::init_kernel_slabs();

    kernel::percpu::init_percpu();
    println!("Per cpu {}", kernel::percpu::tmp());
    drivers::init();

    let mut table = thread_table::thread_table_mut();
    table
        .new_kernel_thread("kernel thread", idle_thread, ())
        .expect("Failed to create kernel thread");

    drop(table);

    sched::init_userspace();

    bring_up_cpus();

    loop {}
}

#[no_mangle]
extern "C" fn cpu_reset() -> ! {
    println!("Cpu {} started!", arch::cpuid::current_cpu());

    loop {}
}
