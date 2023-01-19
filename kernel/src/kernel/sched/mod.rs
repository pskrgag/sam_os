pub mod run_queue;

use crate::{
    arch::{self, irq, regs::Context},
    kernel::elf::parse_elf,
    kernel::threading::thread_ep::idle_thread,
    kernel::threading::{
        thread::ThreadState,
        ThreadRef,
    },
};

use run_queue::RUN_QUEUE;

extern "C" {
    fn switch_to(from: *mut Context, to: *const Context);
}

static INIT: &[u8] = include_bytes!("../../../../target/aarch64-unknown-none-softfloat/debug/init");

// Simple, Simple, Simple
//
// On cpu reset on non-boot cpus we need any sp, so we
// steal sp from per-cpu idle thread
#[no_mangle]
pub static mut IDLE_THREAD_STACK: [usize; 2] = [0, 0];

#[inline]
pub fn current() -> Option<ThreadRef> {
    None
}

pub unsafe fn run() {
    let rq = RUN_QUEUE.per_cpu_var_get().get();

    if rq.empty() {
        return;
    }

    let cur = current();

    if let Some(c) = cur {
        let mut cur = c.write();

        if cur.state() != ThreadState::NeedResched {
            return;
        }

        let next = rq.pop().unwrap();

        println!("Switching to {} --> {}", cur.id(), next.read().id());

        let mut next = next.write();

        let ctx = cur.ctx_mut() as *mut _;
        let ctx_next = next.ctx_mut() as *const _;

        drop(cur);

        next.set_state(ThreadState::Running);

        drop(next);

        rq.add(c);

        irq::enable_all();
        switch_to(ctx, ctx_next);
        irq::disable_all();
    } else {
        let mut ctx = Context::default(); // tmp storage
        let next = rq.pop().unwrap();
        let mut next = next.write();

        next.set_state(ThreadState::Running);

        let next_ctx = next.ctx_mut() as *const _;

        drop(next);

        irq::enable_all();
        switch_to(&mut ctx as *mut _, next_ctx);
        irq::disable_all();
    }
}

// ToDo: any idea fow to fix it?
pub fn init_userspace() {
    // let data = parse_elf(INIT).expect("Failed to parse elf");

    // thread_table_mut()
    //     .new_user_thread("init", data, INIT)
    //     .expect("Failed to run user thread");
}

pub fn init_idle() {
    // let mut table = thread_table::thread_table_mut();

    // for i in 0..arch::NUM_CPUS {
    //     let head = table
    //         .new_idle_thread("idle thread", idle_thread, (), i)
    //         .expect("Failed to create kernel thread")
    //         .read()
    //         .stack_head()
    //         .unwrap();

    //     unsafe {
    //         use crate::mm::types::PhysAddr;
    //         IDLE_THREAD_STACK[i] = PhysAddr::from(head).get();
    //     }
    // }
}
