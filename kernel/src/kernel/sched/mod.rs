use crate::kernel::object::thread_object::Thread;
use crate::percpu_global;
use crate::{arch::irq::interrupts, arch::regs::Context, kernel::tasks::thread::ThreadState};
#[cfg(not(test))]
use crate::{kernel::elf::parse_initial_task, kernel::tasks::task::init_task};
use alloc::sync::Arc;
use run_queue::RunQueue;

pub mod run_queue;

unsafe extern "C" {
    fn switch_to(from: *mut Context, to: *const Context);
}

// Simple, Simple, Simple
//
// On cpu reset on non-boot cpus we need any sp, so we
// steal sp from per-cpu idle thread
#[unsafe(no_mangle)]
pub static mut IDLE_THREAD_STACK: [usize; 2] = [0, 0];

#[inline]
pub fn current() -> Option<Arc<Thread>> {
    crate::arch::current::get_current()
}

pub struct Scheduler {
    rq: RunQueue,
}

percpu_global!(
    pub static SCHEDULER: Scheduler = Scheduler::new();
);

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            rq: RunQueue::new(),
        }
    }

    pub fn schedule(&mut self) {
        // Fix up queue based on recent events
        self.rq.move_by_pred(|x| x.state() == ThreadState::Running);

        if let Some(mut cur) = current() {
            match cur.state() {
                ThreadState::Running => return, // Just timer tick
                ThreadState::NeedResched => self.rq.add_running(cur.clone()),
                ThreadState::WaitingMessage => self.rq.add_sleeping(cur.clone()),
                _ => panic!("Running scheduler in unexected state"),
            }

            if let Some(mut next) = self.rq.pop_running() {
                next.set_state(ThreadState::Running);

                unsafe {
                    let ctx = cur.ctx_mut();
                    let next_clone = next.clone();
                    let ctx_next = next.ctx_mut();

                    crate::arch::current::set_current(next_clone);

                    switch_to(ctx as _, ctx_next as _);
                }
            } else if cur.state() == ThreadState::WaitingMessage {
                panic!("WTF");
            }
        } else {
            // If there is nothing to switch to, then do nothing
            if self.rq.empty() {
                return;
            }

            let mut ctx = Context::default(); // tmp storage
            let mut next = self
                .rq
                .pop_running()
                .expect("Rq must not be empty at that moment");
            let next_clone = next.clone();

            next.set_state(ThreadState::Running);
            let next_ctx = unsafe { next.ctx_mut() };

            crate::arch::current::set_current(next_clone);

            // We come here only for 1st process, so we need to turn on irqs
            unsafe {
                interrupts::enable_all();
                switch_to(&mut ctx as *mut _, next_ctx as *const _);
            }
            panic!("Should not reach here");
        }
    }

    pub fn add_thread(&mut self, t: Arc<Thread>) {
        match t.state() {
            ThreadState::Running => self.rq.add_running(t),
            ThreadState::WaitingMessage => self.rq.add_sleeping(t),
            _ => panic!("Called on wrong thread state"),
        }
    }
}

pub fn run() {
    let scheduler = SCHEDULER.per_cpu_var_get_mut();

    scheduler.schedule();
}

pub fn add_thread(t: Arc<Thread>) {
    let scheduler = SCHEDULER.per_cpu_var_get_mut();

    scheduler.add_thread(t);
}

pub fn init_userspace(_prot: &loader_protocol::LoaderArg) {
    #[cfg(not(test))]
    {
        let prot = _prot;

        let data = parse_initial_task(prot).unwrap();
        let init_task = init_task();

        let init_thread = Thread::new(init_task.clone(), 0);

        let init_vms = init_task.vms();

        for mut i in data.regions {
            println!("{:?} {:?} {:?}", i.va, i.pa, i.tp);

            i.va.align_page();
            i.pa.align_page();
            init_vms.vm_map(i.va, i.pa, i.tp).expect("Failed to map");
        }

        init_thread.init_user(data.ep);

        init_task.add_initial_thread(init_thread, rtl::handle::HANDLE_INVALID);
        init_task.start_inner();
    }
}
