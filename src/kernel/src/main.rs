#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_macros)]
#![allow(special_module_name)]
#![feature(int_roundings)]
#![feature(allocator_api)]
#![feature(get_mut_unchecked)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

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

#[cfg(test)]
#[macro_use]
pub mod tests;

static SAMOS_BANNER: &str = "
(  ____ \\(  ___  )(       )  (  ___  )(  ____ \\
| (    \\/| (   ) || () () |  | (   ) || (    \\/
| (_____ | (___) || || || |  | |   | || (_____ 
(_____  )|  ___  || |(_)| |  | |   | |(_____  )
      ) || (   ) || |   | |  | |   | |      ) |
/\\____) || )   ( || )   ( |  | (___) |/\\____) |
\\_______)|/     \\||/     \\|  (_______)\\_______)
                                               
";

/* At this point we have:
 *
 *      1) MMU is turned on
 *      2) MMMIO is mapped as 1 to 1
 *      3) 0xffffffffc0000000 and load_addr are mapped to load_addr via 1GB block
 */
#[no_mangle]
extern "C" fn start_kernel() -> ! {
    println!("Starting kernel...");
    arch::irq::handlers::set_up_vbar();

    // allocators + paging
    mm::init();

    // --- Kernel is fine grained mapped ---
    // all wild accesses will cause exception

    kernel::percpu::init_percpu();

    drivers::init();

    print!("{}", SAMOS_BANNER);

    #[cfg(test)]
    #[allow(clippy::empty_loop)]
    {
        test_main();
        println!("Testing finishes!");
        loop {}
    }

    #[cfg(not(test))]
    #[allow(clippy::empty_loop)]
    {
        sched::init_userspace();

        // arch::smp::bring_up_cpus();

        loop {}
    }
}

#[no_mangle]
extern "C" fn cpu_reset() -> ! {
    println!("Cpu {} started!", arch::cpuid::current_cpu());

    arch::irq::handlers::set_up_vbar();

    unsafe {
        arch::irq::enable_all();
    }

    drivers::timer::init_secondary();

    /*
     * Runqueue for current cpu should already contain
     * idle thread, so just loop until timer irq
     */

    #[allow(clippy::empty_loop)]
    loop {}
}
