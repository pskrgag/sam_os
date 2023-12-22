#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_trait_impl)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_macros)]
#![allow(special_module_name)]
#![feature(int_roundings)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]

extern crate alloc;

#[macro_use]
mod lib;

mod arch;
#[macro_use]
mod kernel;
mod drivers;
mod mm;
mod panic;

use kernel::sched;
pub use lib::printf;

use crate::kernel::tasks::task;

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

    // allocators + paging
    mm::init();

    // --- Kernel is fine grained mapped ---
    // all wild accesses will cause exception

    kernel::percpu::init_percpu();

    task::init_kernel_task();
    sched::init_idle();
    sched::init_userspace();

    // -- Scheduler must be initialized at that point
    drivers::init();

    // arch::smp::bring_up_cpus();

    loop {}
}

#[no_mangle]
extern "C" fn cpu_reset() -> ! {
    println!("Cpu {} started!", arch::cpuid::current_cpu());

    arch::interrupts::set_up_vbar();

    unsafe {
        arch::irq::enable_all();
    }

    drivers::timer::init_secondary();

    /*
     * Runqueue for current cpu should already contain
     * idle thread, so just loop until timer irq
     */

    loop {}
}
