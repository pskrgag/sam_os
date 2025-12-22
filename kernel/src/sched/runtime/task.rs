use crate::sync::Spinlock;
use crate::tasks::thread::Thread;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use rtl::error::ErrorType;

pub struct Task {
    future: Spinlock<Pin<Box<dyn Future<Output = ()>>>>,
    thread: Arc<Thread>,
}

impl Task {
    pub fn new(
        future: impl Future<Output = ()> + 'static,
        thread: Arc<Thread>,
    ) -> Result<Self, ErrorType> {
        Ok(Self {
            future: Spinlock::new(Box::into_pin(
                Box::try_new(future).map_err(|_| ErrorType::NoMemory)?,
            )),
            thread,
        })
    }

    pub fn poll(&self, ctx: &mut Context) -> Poll<()> {
        self.future.lock().as_mut().poll(ctx)
    }

    pub fn thread(&self) -> &Arc<Thread> {
        &self.thread
    }
}
