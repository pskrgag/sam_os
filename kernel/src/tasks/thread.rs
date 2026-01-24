use super::task::Task;
use crate::arch::regs::Context;
use crate::object::KernelObjectBase;
use crate::sched::spawn;
use crate::sync::Spinlock;
use crate::tasks::task::kernel_task;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context as PollContext, Poll, Waker};
use core::time::Duration;
use hal::address::*;
use hal::arch::PAGE_SIZE;
use rtl::error::ErrorType;
use rtl::linker_var;
use rtl::signal::Signal;
use rtl::vmm::MappingType;

const USER_THREAD_STACK_PAGES: usize = 2000;
const KERNEL_STACK_PAGES: usize = 100;
const RR_TICKS: usize = 10;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadType {
    Undef,
    Kernel,
    User,
}

pub struct ThreadInner {
    arch_ctx: Option<Context>,
    stack: MemRange<VirtAddr>,
    waker: Option<Waker>,
}

impl ThreadInner {
    pub fn new(stack: MemRange<VirtAddr>) -> Self {
        Self {
            arch_ctx: None,
            stack,
            waker: None,
        }
    }

    pub fn set_waker(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }

    pub fn wake(&mut self) {
        if let Some(x) = self.waker.take() {
            x.wake()
        }
    }

    pub fn init_context(&mut self, ep: VirtAddr, user_stack: VirtAddr, arg: usize) {
        self.arch_ctx = Some(Context::new(ep, user_stack, arg));
    }

    pub fn take_context(&mut self) -> Option<Context> {
        self.arch_ctx.take()
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.arch_ctx = Some(ctx)
    }
}

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
    Event = 3,
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ThreadRawState(usize);

impl From<ThreadRawState> for usize {
    fn from(state: ThreadRawState) -> usize {
        state.0
    }
}

pub struct Thread {
    id: u16,
    task: Weak<Task>,
    preemtion: AtomicUsize,
    inner: Spinlock<ThreadInner>,
    base: KernelObjectBase,
    state: AtomicUsize,
    pub ticks: AtomicUsize,
    preemtion_counter: AtomicUsize,
}

crate::kernel_object!(Thread, Signal::None.into());

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
    pub async fn new_user(task: Arc<Task>, id: u16) -> Option<Arc<Thread>> {
        let kernel_stack = kernel_task()
            .vms()
            .vm_allocate(KERNEL_STACK_PAGES * PAGE_SIZE, MappingType::Data)
            .await
            .expect("Failed to allocate kernel stack");

        Arc::try_new(Self {
            id,
            inner: Spinlock::new(ThreadInner::new(MemRange::new(
                kernel_stack,
                KERNEL_STACK_PAGES * PAGE_SIZE,
            ))),
            ticks: RR_TICKS.into(),
            preemtion: AtomicUsize::new(0),
            task: Arc::downgrade(&task),
            state: AtomicUsize::new(
                ThreadRawState::from_raw_parts(ThreadState::Initialized, ThreadSleepReason::None)
                    .into(),
            ),
            base: KernelObjectBase::new(),
            preemtion_counter: 0.into(),
        })
        .ok()
    }

    pub fn initial() -> Option<Arc<Thread>> {
        unsafe extern "C" {
            static __STACK_START: usize;
        }

        let stack_start = linker_var!(__STACK_START);

        Arc::try_new(Self {
            id: 0,
            inner: Spinlock::new(ThreadInner::new(MemRange::new(stack_start.into(), 0x50000))),
            ticks: RR_TICKS.into(),
            preemtion: AtomicUsize::new(0),
            task: Arc::downgrade(&kernel_task()),
            state: AtomicUsize::new(
                ThreadRawState::from_raw_parts(ThreadState::Running, ThreadSleepReason::None)
                    .into(),
            ),
            base: KernelObjectBase::new(),
            preemtion_counter: 0.into(),
        })
        .ok()
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn task(&self) -> Arc<Task> {
        self.task.upgrade().unwrap()
    }

    pub async fn init_user(self: &Arc<Thread>, ep: VirtAddr, args: Option<usize>) {
        let task = self.task.upgrade().unwrap();
        let vms = task.vms();
        let user_stack = vms
            .vm_allocate(USER_THREAD_STACK_PAGES * PAGE_SIZE, MappingType::Data)
            .await
            .expect("Failed to allocate user stack");

        let mut inner = self.inner.lock();

        inner.init_context(
            ep,
            VirtAddr::from(user_stack.bits() + USER_THREAD_STACK_PAGES * PAGE_SIZE),
            args.unwrap_or(0),
        );
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

    pub fn start(self: &Arc<Self>) -> Result<(), ErrorType> {
        use crate::sched::userspace_loop;

        self.set_running();
        spawn(
            Box::into_pin(
                Box::try_new(userspace_loop(self.clone())).map_err(|_| ErrorType::NoMemory)?,
            ),
            self.clone(),
        )
    }

    fn request_resched(self: &Arc<Self>) {
        self.set_state(ThreadState::NeedResched, ThreadSleepReason::None);

        // TODO: better move to scheduler
        self.ticks.store(RR_TICKS, Ordering::Relaxed);
    }

    fn disable_preemtion(self: &Arc<Self>) {
        self.preemtion_counter.fetch_add(1, Ordering::Relaxed);
    }

    fn enable_preemtion(self: &Arc<Self>) {
        let enable = self.preemtion_counter.fetch_sub(1, Ordering::Relaxed) == 1;

        if enable {
            self.request_resched();
        }
    }

    pub fn is_preemtion_enabled(self: &Arc<Self>) -> bool {
        self.preemtion_counter.load(Ordering::Relaxed) == 0
    }

    pub fn with_disabled_preemption<F: FnOnce()>(self: &Arc<Self>, f: F) {
        self.disable_preemtion();
        f();
        self.enable_preemtion();
    }

    pub async fn context(self: &Arc<Self>) -> Context {
        struct RunnableChecker {
            thread: Arc<Thread>,
        }

        impl Future for RunnableChecker {
            type Output = Context;

            fn poll(self: Pin<&mut Self>, cx: &mut PollContext) -> Poll<Self::Output> {
                match self.thread.state() {
                    ThreadState::Running => {
                        Poll::Ready(self.thread.inner.lock().take_context().unwrap())
                    }
                    ThreadState::NeedResched => {
                        self.thread.set_running();
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    _ => todo!(),
                }
            }
        }

        match self.state() {
            ThreadState::Running => self.inner.lock().take_context().unwrap(),
            ThreadState::NeedResched => {
                RunnableChecker {
                    thread: self.clone(),
                }
                .await
            }
            _ => todo!(),
        }
    }

    pub fn update_context(self: &Arc<Self>, ctx: Context) {
        let mut inner = self.inner.lock();

        inner.set_context(ctx)
    }

    pub fn tick(self: &Arc<Thread>) {
        let old = self
            .ticks
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
                if x == 0 {
                    None
                } else {
                    Some(x - 1)
                }
            });

        // old.is_err() means thread run out of quantum. When it will be re-enabled thread
        // will be punished by force reschedule
        if old.is_err() && self.is_preemtion_enabled() {
            self.request_resched();
        }
    }

    pub async fn self_yield() {
        struct Yield;

        impl Future for Yield {
            type Output = ();

            fn poll(self: Pin<&mut Self>, _cx: &mut PollContext) -> Poll<Self::Output> {
                // Drop to the scheduler
                // TODO: (reset ticks)
                Poll::Ready(())
            }
        }

        Yield.await
    }

    pub fn state(self: &Arc<Thread>) -> ThreadState {
        ThreadRawState(self.state.load(Ordering::Relaxed)).get_state()
    }

    pub async fn sleep_for(dl: Duration) -> Result<(), ErrorType> {
        use crate::sched::timer::{set_timer, time_since_start};

        struct Sleep {
            dl: Duration,
            diff: Duration,
        }

        impl Future for Sleep {
            type Output = Result<(), ErrorType>;

            fn poll(self: Pin<&mut Self>, cx: &mut PollContext) -> Poll<Self::Output> {
                let waker = cx.waker().clone();

                if time_since_start() >= self.dl {
                    Poll::Ready(Ok(()))
                } else {
                    set_timer(
                        self.diff,
                        alloc::boxed::Box::try_new(move || waker.wake_by_ref())
                            .map_err(|_| ErrorType::NoMemory)?,
                    );
                    Poll::Pending
                }
            }
        }

        Sleep {
            dl: time_since_start() + dl,
            diff: dl,
        }
        .await
    }
}
