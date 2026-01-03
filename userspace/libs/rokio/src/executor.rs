use alloc::boxed::Box;
use alloc::vec::Vec;
use async_task::{spawn, Runnable};
use core::future::Future;
use core::pin::Pin;
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

    pub fn spawn<F: Future<Output = ()> + 'static>(&self, f: F) {
        let (runnable, task) =
            unsafe { async_task::spawn_unchecked(f, |runnable| self.runnable.push(runnable)) };

        self.runnable.push(runnable);
    }

    fn poll_runnable(&'static self) {
        while let Some(task) = self.runnable.pop() {
            task.run();
        }
    }

    pub(crate) fn add_wait(&self, w: Waiter) {
        self.waiting.push(w);
    }

    fn wait(&self) -> Result<(), ErrorType> {
        let mut wait_entries = Vec::new();

        while let Some(entry) = self.waiting.pop() {
            wait_entries.push(WaitEntry {
                handle: entry.handle,
                waitfor: entry.waitfor,
                pendind: Signal::None.into(),
                context: entry.waker.data() as usize,
                context1: unsafe { entry.waker.vtable() as *const _ as usize },
            })
        }

        Syscall::object_wait_many(&mut wait_entries)?;

        for entry in wait_entries {
            let waker = unsafe {
                Waker::new(
                    entry.context as _,
                    &*(entry.context1 as *const RawWakerVTable),
                )
            };

            if *(entry.pendind & entry.waitfor) != 0 {
                waker.wake();
            } else {
                self.waiting.push(Waiter {
                    waker,
                    waitfor: entry.waitfor,
                    handle: entry.handle,
                });
            }
        }

        Ok(())
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
