pub mod entity;
pub mod run_queue;

use crate::{
    arch::{irq, regs::Context},
    kernel::elf::parse_elf,
    kernel::threading::{
        thread::ThreadState,
        thread_table::{thread_table, thread_table_mut},
        ThreadRef,
    },
};
use run_queue::RUN_QUEUE;

extern "C" {
    fn switch_to(from: *mut Context, to: *const Context);
}

static INIT: &[u8] = include_bytes!("../../../../target/aarch64-unknown-none-softfloat/debug/init");

#[inline]
pub fn current() -> Option<ThreadRef> {
    let id = RUN_QUEUE.get().current_id()?;
    thread_table().thread_by_id(id)
}

pub unsafe fn run() {
    let rq = RUN_QUEUE.get();
    let cur = current();
    let next = rq.pop();

    if next.is_none() {
        return;
    }

    let next = next.unwrap();

    if let Some(c) = cur {
        let mut cur = c.write();

        if cur.state() != ThreadState::NeedResched {
            return;
        }

        println!(
            "Switching to {} --> {}",
            cur.id(),
            next.thread().read().id()
        );

        let mut next = next.thread().write();

        let ctx = cur.ctx_mut() as *mut _;
        let ctx_next = next.ctx_mut() as *const _;

        drop(cur);
        drop(next);

        rq.add(c);

        irq::enable_all();
        switch_to(ctx, ctx_next);
        irq::disable_all();
    } else {
        let mut ctx = Context::default(); // tmp storage
        let next = next.thread().write().ctx_mut() as *const _;

        irq::enable_all();
        switch_to(&mut ctx as *mut _, next);
        irq::disable_all();
    }
}

// ToDo: any idea fow to fix it?
pub fn init_userspace() {
    let data = parse_elf(INIT).expect("Failed to parse elf");

    thread_table_mut()
        .new_user_thread("init", data, INIT)
        .expect("Failed to run user thread");
}
