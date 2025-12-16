use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::kernel::locking::spinlock::Spinlock;

pub struct Task {
    future: Spinlock<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            future: Spinlock::new(Box::pin(future))
        }
    }

    pub fn poll(&self, ctx: &mut Context) -> Poll<()> {
        self.future.lock().as_mut().poll(ctx)
    }
}
