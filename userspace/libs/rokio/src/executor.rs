use alloc::vec::Vec;
use async_task::Runnable;
use core::future::Future;
use core::mem::forget;
use core::task::{RawWakerVTable, Waker};
use crossbeam::queue::SegQueue;
use libc::syscalls::Syscall;
use rtl::error::ErrorType;
use rtl::handle::Handle as RawHandle;
use rtl::signal::{Signal, Signals, WaitEntry};
use spin::lazy::Lazy;

extern crate alloc;

static CURRENT_RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new());

/// Async runtime on top of SAMOS ports
pub struct Runtime {
    runnable: SegQueue<Runnable>,
    waiting: SegQueue<Waiter>,
}

pub(crate) struct Waiter {
    handle: RawHandle,
    waitfor: Signals,
    waker: Waker,
}

impl Waiter {
    pub fn new(handle: RawHandle, waitfor: Signals, waker: Waker) -> Self {
        Self {
            handle,
            waker,
            waitfor,
        }
    }
}

impl Runtime {
    /// Constructs new runtime
    pub fn new() -> Self {
        Self {
            runnable: SegQueue::new(),
            waiting: SegQueue::new(),
        }
    }

    pub fn spawn<F: Future + 'static>(&'static self, f: F)
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
                context: entry.waker.data() as usize,
                context1: entry.waker.vtable() as *const _ as usize,
            };
            wait_entries.push(we);

            // Waker was disassembled. Don't drop the reference here
            forget(entry.waker);
        }

        if wait_entries.len() == 0 {
            return Ok(0);
        }

        Syscall::object_wait_many(&mut wait_entries)?;

        let mut waked = 0;

        for entry in wait_entries {
            let waker = unsafe {
                Waker::new(
                    entry.context as _,
                    &*(entry.context1 as *const RawWakerVTable),
                )
            };

            if *(entry.pendind & entry.waitfor) != 0 {
                waker.wake_by_ref();
                waked += 1;
            } else {
                self.waiting.push(Waiter {
                    waker,
                    waitfor: entry.waitfor,
                    handle: entry.handle,
                });
            }
        }

        Ok(waked)
    }

    pub fn run(&'static self) {
        loop {
            // Poll ready tasks
            self.poll_runnable();

            // Wait for events to occur
            self.wait().unwrap();
        }
    }
}

pub(crate) fn current_runtime() -> &'static Runtime {
    &*CURRENT_RUNTIME
}

pub fn spawn<F: Future + 'static + Send>(f: F)
where
    F::Output: Send,
{
    CURRENT_RUNTIME.spawn(f)
}

// TODO: add back + Send when I figure out wtf rust wants from me
pub fn block_on<F: Future + 'static>(f: F)
where
    F::Output: Send,
{
    CURRENT_RUNTIME.spawn(f);
    CURRENT_RUNTIME.run();
}
