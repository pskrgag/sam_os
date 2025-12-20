use crate::arch::regs::Context;
use crate::kernel::locking::spinlock::Spinlock;
use crate::kernel::object::task_object::Task;
use crate::kernel::object::KernelObjectBase;
use crate::kernel::sched::spawn;
use crate::kernel::tasks::task::kernel_task;
use crate::kernel::tasks::thread::ThreadInner;
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context as PollContext, Poll};
use core::time::Duration;
use hal::address::*;
use hal::arch::PAGE_SIZE;
use object_lib::object;
use rtl::linker_var;
use rtl::vmm::MappingType;

const USER_THREAD_STACK_PAGES: usize = 100;
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
    Event = 3,
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
    base: KernelObjectBase,
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
    pub fn new_user(task: Arc<Task>, id: u16) -> Option<Arc<Thread>> {
        let kernel_stack = kernel_task()
            .vms()
            .vm_allocate(KERNEL_STACK_PAGES * PAGE_SIZE, MappingType::Data)
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
        })
        .ok()
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn task(&self) -> Arc<Task> {
        self.task.upgrade().unwrap()
    }

    pub fn init_user(self: &Arc<Thread>, ep: VirtAddr, args: Option<[usize; 3]>) {
        let task = self.task.upgrade().unwrap();
        let vms = task.vms();
        let user_stack = vms
            .vm_allocate(USER_THREAD_STACK_PAGES * PAGE_SIZE, MappingType::Data)
            .expect("Failed to allocate user stack");

        let mut inner = self.inner.lock();

        inner.init_context(
            ep,
            VirtAddr::from(user_stack.bits() + USER_THREAD_STACK_PAGES * PAGE_SIZE),
            args.unwrap_or([0; 3]),
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

    pub fn start(self: &Arc<Self>) {
        use crate::kernel::sched::userspace_loop;

        self.set_running();
        spawn(
            alloc::boxed::Box::pin(userspace_loop(self.clone())),
            self.clone(),
        );
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
        let old = self.ticks.fetch_sub(1, Ordering::Relaxed);

        if old == 0 {
            self.set_state(ThreadState::NeedResched, ThreadSleepReason::None);

            // TODO: better move to scheduler
            self.ticks.store(RR_TICKS, Ordering::Relaxed);
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

    pub fn wake(self: &Arc<Thread>) {
        self.set_state(ThreadState::Running, ThreadSleepReason::None);
    }

    pub fn state(self: &Arc<Thread>) -> ThreadState {
        ThreadRawState(self.state.load(Ordering::Relaxed)).get_state()
    }

    pub async fn sleep_for(dl: Duration) {
        use crate::kernel::sched::timer::{time_since_start, TIMER_QUEUE};

        struct Sleep {
            dl: Duration,
            diff: Duration,
        }

        impl Future for Sleep {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut PollContext) -> Poll<Self::Output> {
                let waker = cx.waker().clone();

                if time_since_start() >= self.dl {
                    Poll::Ready(())
                } else {
                    TIMER_QUEUE.lock().set_timer(
                        self.diff,
                        alloc::boxed::Box::new(move || waker.wake_by_ref()),
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
