use super::run_queue::RunQueue;
use super::task::Task;
use crate::tasks::thread::Thread;
use alloc::sync::Arc;
use core::task::{Context, Poll};
use rtl::error::ErrorType;

pub struct Executor {
    rq: RunQueue,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            rq: RunQueue::new(),
        }
    }

    pub fn add<F: Future<Output = ()> + 'static>(
        &mut self,
        future: F,
        thread: Arc<Thread>,
    ) -> Result<(), ErrorType> {
        self.rq.add(Task::new(future, thread)?)
    }

    pub fn run(&mut self) {
        use crate::sched::current::set_current;

        for task_ref in self.rq.tasks() {
            let mut ctx = Context::from_waker(&task_ref.waker);
            let thread = task_ref.task.thread();

            set_current(thread.clone());
            thread.task().vms().switch_to();

            match task_ref.task.poll(&mut ctx) {
                Poll::Ready(_) => {}
                Poll::Pending => {
                    // Noting to do
                }
            }

            // Thread should not return to executor with disabled preemption
            assert!(thread.is_preemtion_enabled());
        }
    }
}
