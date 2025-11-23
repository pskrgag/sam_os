use crate::kernel::object::task_object::Task;
use crate::kernel::sched::add_thread;
use crate::kernel::tasks::task::kernel_task;
use crate::kernel::tasks::thread::ThreadInner;
use crate::{arch::regs::Context, kernel::locking::spinlock::Spinlock};
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::sync::atomic::{AtomicUsize, Ordering};
use object_lib::object;
use hal::arch::PAGE_SIZE;
use hal::address::*;
use rtl::vmm::MappingType;

const USER_THREAD_STACK_PAGES: usize = 50;
const KERNEL_STACK_PAGES: usize = 100;
const RR_TICKS: usize = 10;

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(usize)]
pub enum ThreadState {
    Initialized = 0,
    Running = 1,
    Sleeping = 2,
    NeedResched = 3,
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(usize)]
pub enum ThreadSleepReason {
    None = 0,
    Mutex = 1,
    WaitQueue = 2,
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ThreadRawState(usize);

impl Into<usize> for ThreadRawState {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(object)]
pub struct Thread {
    id: u16,
    task: Weak<Task>,
    ticks: AtomicUsize,
    preemtion: AtomicUsize,
    inner: Spinlock<ThreadInner>,
    state: AtomicUsize,
}

impl ThreadRawState {
    fn get_state(&self) -> ThreadState {
        unsafe { core::mem::transmute((self.0 >> 16) & 0xFFFF) }
    }

    fn get_sleep_reason(&self) -> ThreadSleepReason {
        unsafe { core::mem::transmute(self.0 & 0xFFFF) }
    }

    fn from_raw_parts(state: ThreadState, sleep: ThreadSleepReason) -> Self {
        Self((state as usize) << 16 | sleep as usize)
    }
}

impl Thread {
    pub fn new(task: Arc<Task>, id: u16) -> Option<Arc<Thread>> {
        Some(
            Arc::try_new(Self {
                id,
                inner: Spinlock::new(ThreadInner::default()),
                ticks: RR_TICKS.into(),
                preemtion: AtomicUsize::new(0),
                task: Arc::downgrade(&task),
                state: AtomicUsize::new(
                    ThreadRawState::from_raw_parts(
                        ThreadState::Initialized,
                        ThreadSleepReason::None,
                    )
                    .into(),
                ),
            })
            .ok()?,
        )
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn task(&self) -> Arc<Task> {
        self.task.upgrade().unwrap()
    }

    pub unsafe fn ctx_mut(self: &mut Arc<Thread>) -> &mut Context {
        unsafe {
            let mut inner = self.inner.lock();
            let r = &mut inner.arch_ctx as *mut Context;

            // TODO: smells like shit
            &mut *r
        }
    }

    pub fn init_user(self: &Arc<Thread>, ep: VirtAddr) {
        let kernel_stack = kernel_task()
            .vms()
            .vm_allocate(KERNEL_STACK_PAGES * PAGE_SIZE, MappingType::Data)
            .expect("Failed to allocate kernel stack");

        let task = self.task.upgrade().unwrap();
        let vms = task.vms();
        let user_stack = vms
            .vm_allocate(USER_THREAD_STACK_PAGES * PAGE_SIZE, MappingType::Data)
            .expect("Failed to allocate user stack");

        let mut inner = self.inner.lock();

        inner.init_user(
            VirtAddr::from(kernel_stack.bits() + KERNEL_STACK_PAGES * PAGE_SIZE),
            ep,
            VirtAddr::from(user_stack.bits() + USER_THREAD_STACK_PAGES * PAGE_SIZE),
            vms.base().bits(),
        );
    }

    pub fn setup_args(&self, args: &[usize]) {
        let mut inner = self.inner.lock();

        inner.setup_args(args);
    }

    fn set_state(self: &Arc<Self>, state: ThreadState, sleep: ThreadSleepReason) {
        self.state.store(
            ThreadRawState::from_raw_parts(state, sleep).into(),
            Ordering::Relaxed,
        );
    }

    pub fn set_running(self: &Arc<Self>) {
        self.set_state(ThreadState::Running, ThreadSleepReason::None);
    }

    pub fn start(self: &Arc<Self>) {
        self.set_running();
        add_thread(self.clone());
    }

    pub fn tick(self: Arc<Thread>) {
        let old = self.ticks.fetch_sub(1, Ordering::Relaxed);

        if old == 0 {
            self.set_state(ThreadState::NeedResched, ThreadSleepReason::None);
            self.ticks.store(RR_TICKS, Ordering::Relaxed);
        }
    }

    pub fn self_yield(self: Arc<Thread>) {
        self.ticks.store(RR_TICKS, Ordering::Relaxed);
        self.set_state(ThreadState::NeedResched, ThreadSleepReason::None);

        crate::sched::run();
    }

    pub fn wake(self: &Arc<Thread>) {
        self.set_state(ThreadState::Running, ThreadSleepReason::None);
    }

    pub fn state(self: &Arc<Thread>) -> ThreadState {
        ThreadRawState(self.state.load(Ordering::Relaxed)).get_state()
    }

    pub fn sleep(self: &Arc<Thread>, why: ThreadSleepReason) {
        self.set_state(ThreadState::Sleeping, why);
        crate::sched::run();
    }
}
