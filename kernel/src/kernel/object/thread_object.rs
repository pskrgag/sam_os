use crate::kernel::object::task_object::Task;
use crate::kernel::sched::run_queue::RUN_QUEUE;
use crate::kernel::tasks::thread::{ThreadInner, ThreadState};
use crate::{
    arch::regs::Context, kernel::locking::spinlock::Spinlock,
    mm::allocators::stack_alloc::StackLayout,
};
use alloc::boxed::Box;
use alloc::sync::Weak;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use object_lib::object;
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;
use rtl::error::ErrorType;

const USER_THREAD_STACK_PAGES: usize = 15;
const KERNEL_STACK_PAGES: usize = 5;
const RR_TICKS: usize = 10;

#[derive(object)]
pub struct Thread {
    type_id: core::any::TypeId,
    id: u16,
    task: Weak<Task>,
    ticks: AtomicUsize,
    inner: Spinlock<ThreadInner>,
}

impl Thread {
    pub fn new(task: Arc<Task>, id: u16) -> Arc<Thread> {
        Arc::new(Self {
            type_id: core::any::TypeId::of::<Self>(),
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

    pub(crate) fn init_kernel<T>(self: &Arc<Thread>, func: fn(T) -> Option<()>, arg: T) {
        use crate::kernel::misc::ref_mut_to_usize;

        let arg = Box::new(arg);
        let stack = StackLayout::new(3).expect("Failed to allocat stack");

        let mut inner = self.inner.lock();

        // TODO(skripkin): clean up heap allocation
        inner.init_kernel(stack, func as usize, ref_mut_to_usize(Box::leak(arg)));
    }

    pub fn init_user(self: &Arc<Thread>, ep: VirtAddr) {
        let kernel_stack = StackLayout::new(KERNEL_STACK_PAGES).expect("Failed to allocate kernel stack");
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

        RUN_QUEUE.per_cpu_var_get().get().add(self.clone());
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
    }

    pub fn set_state(self: &Arc<Thread>, state: ThreadState) {
        self.inner.lock().state = state;
    }

    pub fn state(self: &Arc<Thread>) -> ThreadState {
        self.inner.lock().state
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        todo!()
    }
}
