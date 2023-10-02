use crate::{
    arch::{self, regs::Context},
    mm::allocators::stack_alloc::StackLayout,
    mm::{
        types::{Address, VirtAddr},
    },
    lib::refcounter::RefCounter,
};
use crate::kernel::sched::run_queue::RUN_QUEUE;
use super::task::TaskObjectRef;
use alloc::{boxed::Box};

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

#[derive(object)]
#[repr(C)]
pub struct Thread {
    type_id: core::any::TypeId,
    id: u16,
    arch_ctx: Context,
    state: ThreadState,
    task: TaskObjectRef,
    stack: Option<StackLayout>,
    ticks: usize,
    refcount: RefCounter,
}

impl Thread {
    pub fn new(task: TaskObjectRef, id: u16) -> Box<RwLock<Thread>> {
        Box::new(RwLock::new(Self {
            type_id: core::any::TypeId::of::<Self>(),
            id: id,
            state: ThreadState::Initialized,
            arch_ctx: Context::default(),
            stack: None,
            ticks: RR_TICKS,
            task: task,
            refcount: RefCounter::new(),
        }))
    }

    pub fn stack_head(&self) -> Option<VirtAddr> {
        if let Some(s) = &self.stack {
            Some(s.stack_head())
        } else {
            None
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn ctx_mut(&mut self) -> &mut Context {
        &mut self.arch_ctx
    }

    pub fn ctx(&mut self) -> &Context {
        &self.arch_ctx
    }

    pub(crate) fn init_kernel<T>(&mut self, func: fn(T) -> Option<()>, arg: T) {
        use crate::kernel::misc::ref_mut_to_usize;

        let arg = Box::new(arg);
        let stack = StackLayout::new(3).expect("Failed to allocat stack");

        self.arch_ctx.sp = stack.stack_head().into();
        self.arch_ctx.lr = (kernel_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x19 = ref_mut_to_usize(Box::leak(arg));
        self.arch_ctx.x20 = func as usize;

        self.stack = Some(stack);

        self.state = ThreadState::Running;
    }

    pub fn init_user(&mut self, ep: VirtAddr) {
        let stack = StackLayout::new(3).expect("Failed to allocat stack");
        let vms = self.task.read().vms();
        let mut vms = vms.write();
        let user_stack = vms
            .alloc_user_stack()
            .expect("Failed to allocate user stack");

        self.arch_ctx.x21 = user_stack.bits() + arch::PAGE_SIZE;
        self.arch_ctx.lr = (user_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x20 = ep.bits();
        self.arch_ctx.x19 = stack.stack_head().into();
        self.arch_ctx.x22 = stack.stack_head().into();
        self.arch_ctx.ttbr0 = vms.ttbr0().expect("TTBR0 should be set").get();

        self.stack = Some(stack);

        self.state = ThreadState::Initialized;
    }

    pub fn start(t: RefMut<Thread>) {
        let mut ta = t.write();

        ta.state = ThreadState::Running;
        ta.task.write().add_thread(Arc::downgrade(&t));

        drop(ta);
        RUN_QUEUE.per_cpu_var_get().get().add(t);
    }

    pub fn setup_args(&mut self, args: &[&str]) {
        // SAFETY: thread is not running, so we can assume that user addresses
        // are mapped

        for i in args {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    i.as_bytes().as_ptr(),
                    self.arch_ctx.x19 as *mut _,
                    i.len(),
                );
            }

            self.arch_ctx.x19 += i.len();
            self.arch_ctx.x23 += 1;
        }
    }

    pub fn tick(&mut self) {
        self.ticks -= 1;

        if self.ticks == 0 {
            self.state = ThreadState::NeedResched;
            self.ticks = RR_TICKS;
        }
    }

    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }

    pub fn state(&self) -> ThreadState {
        self.state
    }
}
