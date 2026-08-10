use alloc::sync::Arc;
use alloc::vec::Vec;
use async_task::Runnable;
use core::future::Future;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;
use crossbeam::queue::SegQueue;
use libc::syscalls::Syscall;
use rtl::error::ErrorType;
use rtl::handle::Handle as RawHandle;
use rtl::signal::{Signal, Signals, WaitEntry};
use spin::lazy::Lazy;

static CURRENT_RUNTIME: Lazy<Runtime> = Lazy::new(Runtime::new);

/// Async runtime on top of SAMOS objects
#[derive(Default)]
pub struct Runtime {
    runnable: SegQueue<Runnable>,
    waiting: SegQueue<Waiter>,
}

pub(crate) struct WaiterState {
    completed: AtomicBool,
    waker: Waker,
}

pub(crate) struct Waiter {
    handle: RawHandle,
    waitfor: Signals,
    state: Arc<WaiterState>,
}

impl WaiterState {
    pub fn new(waker: Waker) -> Arc<Self> {
        Arc::new(Self {
            completed: false.into(),
            waker,
        })
    }

    pub fn completed(&self) -> bool {
        self.completed.load(Ordering::Acquire)
    }
}

impl Waiter {
    pub fn new(handle: RawHandle, waitfor: Signals, state: Arc<WaiterState>) -> Self {
        Self {
            handle,
            waitfor,
            state,
        }
    }
}

impl Runtime {
    /// Constructs new runtime
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn<F: Future>(&'static self, f: F)
    where
        F::Output: Send,
    {
        let (runnable, task) = unsafe {
            async_task::spawn_unchecked(f, |runnable: Runnable| self.runnable.push(runnable))
        };

        task.detach();
        runnable.schedule();
    }

    fn poll_runnable(&'static self) {
        while let Some(task) = self.runnable.pop() {
            task.run();
        }
    }

    pub(crate) fn add_wait(&self, w: Waiter) {
        self.waiting.push(w);
    }

    fn wait(&self) -> Result<usize, ErrorType> {
        let mut wait_entries = Vec::new();

        while let Some(entry) = self.waiting.pop() {
            let we = WaitEntry {
                handle: entry.handle,
                waitfor: entry.waitfor,
                pendind: Signal::None.into(),
                context: Arc::into_raw(entry.state) as usize,
                context1: 0,
            };
            wait_entries.push(we);
        }

        if wait_entries.is_empty() {
            return Ok(0);
        }

        Syscall::object_wait_many(&mut wait_entries)?;

        let mut waked = 0;

        for entry in wait_entries {
            let state: Arc<WaiterState> = unsafe { Arc::from_raw(entry.context as *const _) };

            if *(entry.pendind & entry.waitfor) != 0 {
                state.completed.store(true, Ordering::Release);
                state.waker.wake_by_ref();
                waked += 1;
            } else {
                self.waiting.push(Waiter {
                    state,
                    waitfor: entry.waitfor,
                    handle: entry.handle,
                });
            }
        }

        Ok(waked)
    }

    pub fn run(&'static self) {
        while !self.waiting.is_empty() || !self.runnable.is_empty() {
            // Poll ready tasks
            self.poll_runnable();

            // Wait for events to occur
            self.wait().unwrap();
        }
    }
}

pub(crate) fn current_runtime() -> &'static Runtime {
    &CURRENT_RUNTIME
}

pub fn spawn<F: Future + Send + 'static>(f: F)
where
    F::Output: Send,
{
    CURRENT_RUNTIME.spawn(f)
}

// TODO: add back + Send when I figure out wtf rust wants from me
pub fn block_on<F: Future>(f: F)
where
    F::Output: Send,
{
    CURRENT_RUNTIME.spawn(f);
    CURRENT_RUNTIME.run();
}
