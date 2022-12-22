pub mod entity;
pub mod run_queue;

use crate::{
    arch::{irq, regs::Context},
    kernel::threading::{thread::ThreadState, thread_table::thread_table, ThreadRef},
};
use run_queue::RUN_QUEUE;

extern "C" {
    fn switch_to(from: *mut Context, to: *const Context);
}

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

        let ctx = cur.ctx_mut() as *mut _;
        let next = next.thread().write().ctx_mut() as *const _;

        drop(cur);
        rq.add(c);

        irq::enable_all();
        switch_to(ctx, next);
        irq::disable_all();
    } else {
        let mut ctx = Context::default(); // tmp storage
        let next = next.thread().write().ctx_mut() as *const _;

        irq::enable_all();
        switch_to(&mut ctx as *mut _, next);
        irq::disable_all();
    }
}
