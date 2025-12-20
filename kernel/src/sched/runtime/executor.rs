use super::run_queue::RunQueue;
use super::task::Task;
use crate::tasks::thread::Thread;
use crate::tasks::task::kernel_task;
use alloc::sync::Arc;
use core::task::{Context, Poll};
use hal::address::VirtAddr;
use hal::arch::PAGE_SIZE;
use rtl::vmm::MappingType;

const STACK_PAGES: usize = 10;

pub struct Executor {
    rq: RunQueue,
    stack: VirtAddr,
}

impl Executor {
    pub fn new() -> Self {
        let stack = kernel_task()
            .vms()
            .vm_allocate(STACK_PAGES * PAGE_SIZE, MappingType::Data)
            .expect("Failed to allocate stack for executor");

        Self {
            stack,
            rq: RunQueue::new(),
        }
    }

    pub fn add<F: Future<Output = ()> + Send + 'static>(&mut self, future: F, thread: Arc<Thread>) {
        self.rq.add(Task::new(future, thread));
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
                    // Noting to dod
                }
            }
        }
    }
}
