use crate::tasks::thread::Thread;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Task {
    future: Spinlock<Pin<Box<dyn Future<Output = ()> + Send>>>,
    thread: Arc<Thread>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static, thread: Arc<Thread>) -> Self {
        Self {
            future: Spinlock::new(Box::pin(future)),
            thread,
        }
    }

    pub fn poll(&self, ctx: &mut Context) -> Poll<()> {
        self.future.lock().as_mut().poll(ctx)
    }

    pub fn thread(&self) -> &Arc<Thread> {
        &self.thread
    }
}
