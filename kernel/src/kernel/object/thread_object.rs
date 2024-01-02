use crate::kernel::object::task_object::Task;
use crate::kernel::sched::add_thread;
use crate::kernel::tasks::thread::{ThreadInner, ThreadState};
use crate::{
    arch::regs::Context, kernel::locking::spinlock::Spinlock,
    mm::allocators::stack_alloc::StackLayout,
};
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::sync::atomic::{AtomicUsize, Ordering};
use object_lib::object;
use rtl::arch::PAGE_SIZE;
use rtl::error::ErrorType;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

const USER_THREAD_STACK_PAGES: usize = 15;
const KERNEL_STACK_PAGES: usize = 5;
const RR_TICKS: usize = 10;

#[derive(object)]
pub struct Thread {
    id: u16,
    task: Weak<Task>,
    ticks: AtomicUsize,
    inner: Spinlock<ThreadInner>,
}

impl Thread {
    pub fn new(task: Arc<Task>, id: u16) -> Arc<Thread> {
        Arc::new(Self {
            id,
            inner: Spinlock::new(ThreadInner::default()),
            ticks: RR_TICKS.into(),
            task: Arc::downgrade(&task),
        })
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn task(&self) -> Arc<Task> {
        self.task.upgrade().unwrap()
    }

    pub unsafe fn ctx_mut<'a>(self: &'a Arc<Thread>) -> &'a mut Context {
        let mut inner = self.inner.lock();
        let r = &mut inner.arch_ctx as *mut Context;

        // TODO(skripkin): smells like shit
        &mut *r
    }

    pub fn init_user(self: &Arc<Thread>, ep: VirtAddr) {
        let kernel_stack =
            StackLayout::new(KERNEL_STACK_PAGES).expect("Failed to allocate kernel stack");
        let vms = self.task.upgrade().unwrap().vms();
        let user_stack = vms
            .vm_allocate(USER_THREAD_STACK_PAGES * PAGE_SIZE, MappingType::USER_DATA)
            .expect("Failed to allocate user stack");

        let mut inner = self.inner.lock();

        inner.init_user(
            kernel_stack,
            ep,
            VirtAddr::from(user_stack.bits() + USER_THREAD_STACK_PAGES * PAGE_SIZE),
            vms.base().bits(),
        );
    }

    pub fn setup_args(&self, args: &[usize]) {
        let mut inner = self.inner.lock();

        inner.setup_args(args);
    }

    pub fn start(self: &Arc<Self>) {
        self.inner.lock().state = ThreadState::Running;

        add_thread(self.clone());
    }

    pub fn tick(self: Arc<Thread>) {
        let old = self.ticks.fetch_sub(1, Ordering::Relaxed);

        if old == 0 {
            self.inner.lock().state = ThreadState::NeedResched;
            self.ticks.store(RR_TICKS, Ordering::Relaxed);
        }
    }

    pub fn self_yield(self: Arc<Thread>) {
        self.ticks.store(RR_TICKS, Ordering::Relaxed);
        self.set_state(ThreadState::NeedResched);
        crate::sched::run();
    }

    pub fn set_state(self: &Arc<Thread>, state: ThreadState) {
        self.inner.lock().state = state;
    }

    pub fn state(self: &Arc<Thread>) -> ThreadState {
        self.inner.lock().state
    }

    pub fn wait_send(self: &Arc<Thread>) {
        let mut inner = self.inner.lock();

        assert!(inner.state == ThreadState::Running);
        inner.state = ThreadState::WaitingMessage;

        // drop the lock
        drop(inner);

        crate::sched::run();
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        todo!()
    }
}
