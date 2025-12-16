use super::run_queue::RunQueue;
use super::task::Task;
use crate::kernel::tasks::task::kernel_task;
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

    pub fn add<F: Future<Output = ()> + Send + 'static>(&mut self, future: F) {
        self.rq.add(Task::new(future));
    }

    pub fn run(&mut self) {
        for task_ref in self.rq.tasks() {
            let mut ctx = Context::from_waker(&task_ref.waker);

            match task_ref.task.poll(&mut ctx) {
                Poll::Ready(_) => {}
                Poll::Pending => {
                    // Noting to dod
                }
            }
        }
    }
}
