use crate::arch::regs::{Context, TrapReason};
use crate::drivers::irq::irq::irq_dispatch;
use crate::kernel::object::thread_object::Thread;
use crate::kernel::syscalls::do_syscall;
use crate::percpu_global;
use aarch64_cpu::registers::{ELR_EL1, ESR_EL1, FAR_EL1, Readable};
use alloc::sync::Arc;
use core::cell::LazyCell;
use runtime::executor::Executor;

pub mod current;
pub mod runtime;
pub mod ticks;
pub mod timer;

unsafe extern "C" {
    fn switch_to(from: *mut Context, to: *const Context);
}

#[inline]
pub fn current() -> Option<Arc<Thread>> {
    crate::kernel::sched::current::get_current()
}

pub struct Scheduler {
    rq: Executor,
}

percpu_global!(
    pub static SCHEDULER: LazyCell<Scheduler> = LazyCell::new(Scheduler::new);
);

impl Scheduler {
    pub fn new() -> Self {
        Self {
            rq: Executor::new(),
        }
    }
}

pub fn spawn<F: Future<Output = ()> + Send + 'static>(future: F, thread: Arc<Thread>) {
    SCHEDULER.per_cpu_var_get_mut().rq.add(future, thread);
}

pub fn run() {
    SCHEDULER.per_cpu_var_get_mut().rq.run();
}

pub async fn userspace_loop(thread: Arc<Thread>) {
    loop {
        let mut ctx = thread.context().await;

        unsafe {
            ctx.switch();
        }

        match ctx.trap_reason() {
            TrapReason::Syscall => {
                let res = match ctx.try_into() {
                    Ok(args) => do_syscall(args).await,
                    Err(err) => Err(err),
                };
                let res = match res {
                    Ok(res) => res,
                    Err(err) => -(err as isize) as usize,
                };

                ctx.finish_syscall(res);
            }
            TrapReason::Irq => irq_dispatch(),
            _ => todo!(
                "{:?} 0x{:x} 0x{:x} 0x{:x}",
                ctx,
                ELR_EL1.get(),
                ESR_EL1.get(),
                FAR_EL1.get()
            ),
        }

        thread.update_context(ctx);
    }
}
