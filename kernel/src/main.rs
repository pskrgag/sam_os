#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![allow(non_upper_case_globals)]
#![allow(unused_macros)]
#![allow(special_module_name)]
#![feature(int_roundings)]
#![feature(allocator_api)]
#![feature(get_mut_unchecked)]
#![feature(custom_test_frameworks)]
#![allow(unexpected_cfgs)]
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

unsafe extern "C" {
    static __start: usize;
}

#[unsafe(no_mangle)]
extern "C" fn start_kernel(prot: &mut loader_protocol::LoaderArg) -> ! {
    drivers::init_logging(prot);

    println!("Booting kernel...");
    arch::init(prot);

    mm::init(prot);
    kernel::percpu::init_percpu();
    drivers::init(prot);

    print!("{SAMOS_BANNER}");

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
        sched::init_userspace(prot);

        #[allow(clippy::empty_loop)]
        loop {}
    }
}

#[unsafe(no_mangle)]
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
