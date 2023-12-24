use crate::kernel::sched::run_queue::RUN_QUEUE;
use crate::{
    arch::{self, regs::Context},
    kernel::locking::spinlock::{Spinlock},
    mm::allocators::stack_alloc::StackLayout,
    mm::types::{Address, VirtAddr},
    kernel::tasks::task::Task,
};
use shared::vmm::MappingType;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, Ordering};

use object_lib::object;

extern "C" {
    fn kernel_thread_entry_point();
    fn user_thread_entry_point();
}

const RR_TICKS: usize = 10;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadState {
    Initialized,
    Running,
    Sleeping,
    NeedResched,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadType {
    Undef,
    Kernel,
    User,
}

struct ThreadInner {
    arch_ctx: Context,
    pub(crate) state: ThreadState,
    stack: Option<StackLayout>,
}

#[derive(object)]
pub struct Thread {
    type_id: core::any::TypeId,
    id: u16,
    task: Arc<Task>,
    ticks: AtomicUsize,
    inner: Spinlock<ThreadInner>
}

impl ThreadInner {
    pub fn default() -> Self {
        Self {
            state: ThreadState::Initialized,
            arch_ctx: Context::default(),
            stack: None,
        }
    }

    pub fn init_kernel(&mut self, stack: StackLayout, func: usize, arg: usize) {
        self.arch_ctx.sp = stack.stack_head().into();
        self.arch_ctx.lr = (kernel_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x19 = arg;
        self.arch_ctx.x20 = func as usize;

        self.stack = Some(stack);

        self.state = ThreadState::Running;
    }

    pub fn init_user(&mut self, stack: StackLayout, func: VirtAddr, user_stack: VirtAddr, ttbr0: usize) {
        self.arch_ctx.x21 = user_stack.bits() + arch::PAGE_SIZE;
        self.arch_ctx.lr = (user_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x20 = func.bits();
        self.arch_ctx.x19 = stack.stack_head().into();
        self.arch_ctx.x22 = stack.stack_head().into();
        self.arch_ctx.ttbr0 = ttbr0;

        self.stack = Some(stack);

        self.state = ThreadState::Initialized;
   }
}

impl Thread {
    pub fn new(task: Arc<Task>, id: u16) -> Arc<Thread> {
        Arc::new(Self {
            type_id: core::any::TypeId::of::<Self>(),
            id,
            inner: Spinlock::new(ThreadInner::default()),
            ticks: RR_TICKS.into(),
            task,
        })
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn task(&self) -> Arc<Task> {
        self.task.clone()
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
        let stack = StackLayout::new(3).expect("Failed to allocat stack");
        let vms = self.task.vms();
        let user_stack = vms
            .vm_allocate(5 * arch::PAGE_SIZE, MappingType::USER_DATA)
            .expect("Failed to allocate user stack");

        let mut inner = self.inner.lock();

        inner.init_user(stack, ep, user_stack, vms.base().bits());
    }

    pub fn start(self: &Arc<Self>) {
        self.inner.lock().state = ThreadState::Running;
        self.task.add_thread(Arc::downgrade(self));

        RUN_QUEUE.per_cpu_var_get().get().add(self.clone());
    }

    pub fn setup_args(&mut self, _args: &[&str]) {
        // // SAFETY: thread is not running, so we can assume that user addresses
        // // are mapped
        //
        // for i in args {
        //     unsafe {
        //         core::ptr::copy_nonoverlapping(
        //             i.as_bytes().as_ptr(),
        //             self.arch_ctx.x19 as *mut _,
        //             i.len(),
        //         );
        //     }
        //
        //     self.arch_ctx.x19 += i.len();
        //     self.arch_ctx.x23 += 1;
        // }
    }

    pub fn tick(self: Arc<Thread>) {
        let old = self.ticks.fetch_sub(1, Ordering::Relaxed);

        if old == 0 {
            self.inner.lock().state = ThreadState::NeedResched;
            self.ticks.store(RR_TICKS, Ordering::Relaxed);
        }
    }

    pub fn set_state(self: &Arc<Thread>, state: ThreadState) {
        self.inner.lock().state = state;
    }

    pub fn state(self: &Arc<Thread>) -> ThreadState {
        self.inner.lock().state
    }
}
