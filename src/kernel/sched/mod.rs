pub mod entity;
pub mod run_queue;

use crate::{
    arch::{irq, regs::Context},
    kernel::{
        sched::run_queue::RunQueue,
        threading::{thread_table::thread_table, ThreadRef},
    },
};
use run_queue::RUN_QUEUE;

extern "C" {
    fn switch_to(from: *mut Context, to: *const Context);
}

fn current_no_lock(rq: &RunQueue) -> Option<ThreadRef> {
    let id = rq.current_id()?;
    thread_table().thread_by_id(id)
}

pub unsafe fn switch_to_next() {
    let mut rq = RUN_QUEUE.lock();
    let cur = current_no_lock(&rq);
    let next = rq.pop();

    if next.is_none() {
        return;
    }

    let next = next.unwrap();

    if let Some(cur) = cur {
        println!("Switching to {:p}", &*next.thread().read() as *const _);
        let ctx = cur.write().ctx_mut() as *mut _;
        let mut next = next.thread().write().ctx_mut() as *const _;
        rq.add(cur);

        drop(rq);

        irq::enable_all();
        switch_to(ctx, next);
        irq::disable_all();
    } else {
        let mut ctx = Context::default(); // tmp storage
        let mut next = next.thread().write().ctx_mut() as *const _;

        drop(rq);

        irq::enable_all();
        switch_to(&mut ctx as *mut _, next);
        irq::disable_all();
    }
}

pub fn current() -> Option<ThreadRef> {
    current_no_lock(&*RUN_QUEUE.lock())
}
