#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(custom_test_frameworks)]
#![feature(coroutines, coroutine_trait, iter_from_coroutine)]
#![feature(linked_list_cursors)]
#![allow(unexpected_cfgs)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(not(test))]
use crate::{tasks::elf::parse_initial_task, tasks::task::init_task};

extern crate alloc;

#[macro_use]
mod helper;
mod arch;
#[macro_use]
mod smp;
mod drivers;
mod mm;
mod panic;
mod sync;
mod sched;
mod object;
mod syscalls;
mod tasks;

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

#[cfg(not(test))]
pub fn init_userspace(prot: &loader_protocol::LoaderArg) {
    let data = parse_initial_task(prot).unwrap();
    let init_task = init_task();

    let init_vms = init_task.vms();

    for mut i in data.regions {
        i.va.align_page();
        i.pa.align_page();
        init_vms
            .vm_map(Some(i.va), i.pa, i.tp)
            .expect("Failed to map");
    }

    init_task
        .start(data.ep, None)
        .expect("Failed to start first task");
}

#[unsafe(no_mangle)]
extern "C" fn start_kernel(prot: &mut loader_protocol::LoaderArg) -> ! {
    drivers::init_logging(prot);

    println!("Booting kernel...");
    arch::init(prot);

    mm::init(prot);
    smp::init_percpu();
    drivers::init(prot);

    print!("{SAMOS_BANNER}");

    #[cfg(not(test))]
    init_userspace(prot);

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
        sched::run();
        loop {}
    }
}

#[unsafe(no_mangle)]
extern "C" fn cpu_reset() -> ! {
    // println!("Cpu {} started!", arch::cpuid::current_cpu());
    //
    // arch::irq::handlers::set_up_vbar();
    // arm_gic::irq_enable();
    // drivers::timer::init_secondary();
    //
    /*
     * Runqueue for current cpu should already contain
     * idle thread, so just loop until timer irq
     */

    #[allow(clippy::empty_loop)]
    loop {}
}
