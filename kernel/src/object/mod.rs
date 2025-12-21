use crate::sync::Mutex;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::any::Any;
use core::future::Future;
use core::ops::Deref;
use core::pin::Pin;
use core::task::{Context, Poll};
use rtl::signal::{Signal, Signals};

pub mod capabilities;
pub mod handle;
pub mod handle_table;

pub mod factory_object;
pub mod port_object;

pub type Observer = Box<dyn Fn(Signals) -> bool + Send>;

#[derive(Default)]
struct KernelObjectBaseInner {
    signals: Signals,
    observers: Vec<Observer>,
}

impl KernelObjectBaseInner {
    fn add_observer(&mut self, obs: Observer) {
        if !obs(self.signals) {
            self.observers.push(obs);
        }
    }
}

pub struct KernelObjectBase(Mutex<KernelObjectBaseInner>);

impl KernelObjectBase {
    pub fn new() -> Self {
        Self(Mutex::new(KernelObjectBaseInner::default()))
    }

    pub fn signals(&self) -> Signals {
        self.0.lock().signals
    }

    pub fn add_observer(&self, obs: Observer) {
        self.0.lock().add_observer(obs);
    }

    pub async fn wait_signal(&self, sig: Signals) {
        struct Wait<'a> {
            sig: Signals,
            base: &'a KernelObjectBase,
        }

        impl Future for Wait<'_> {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut inner = self.base.0.lock();

                if inner.signals.contains(self.sig) {
                    Poll::Ready(())
                } else {
                    let waker = cx.waker().clone();
                    let wait_sig = self.sig;

                    inner.add_observer(Box::new(move |sig| {
                        if sig.contains(wait_sig) {
                            waker.wake_by_ref();
                            true
                        } else {
                            false
                        }
                    }));

                    Poll::Pending
                }
            }
        }

        Wait { base: self, sig }.await
    }

    pub fn signal(&self, sig: Signals) {
        let mut inner = self.0.lock();
        inner.signals |= sig;

        let signals = inner.signals;
        inner.observers.retain(|x| !(x)(signals));
    }
}

pub struct WaitManyArg {
    pub obj: Arc<dyn KernelObject + Send>,
    pub waitfor: Signals,
    pub pending: Signals,
}

pub async fn wait_many(entries: &mut Vec<WaitManyArg>) {
    struct WaitMany<'a> {
        entries: &'a Vec<WaitManyArg>,
        polled: bool,
    }

    impl Future for WaitMany<'_> {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if !self.polled {
                for i in self.entries {
                    let waitfor = i.waitfor;
                    let waker = cx.waker().clone();

                    i.obj.add_observer(Box::new(move |sig| {
                        if sig.contains(waitfor) {
                            waker.wake_by_ref();
                            true
                        } else {
                            false
                        }
                    }));
                }

                self.get_mut().polled = true;
                Poll::Pending
            } else {
                for i in self.entries {
                    if *(i.obj.signals() & i.waitfor) != 0 {
                        return Poll::Ready(());
                    }
                }

                Poll::Pending
            }
        }
    }

    // Wait for any object to signal
    WaitMany {
        entries,
        polled: false,
    }
    .await;

    for entry in entries {
        entry.pending = entry.obj.signals() & entry.waitfor;
    }
}

// All exposed kernel objects must be derived from this trait
pub trait KernelObject: Send + Deref<Target = KernelObjectBase> {
    /// Expose yourself as Any to allow storing in capability table
    fn as_any(&self) -> &dyn Any;

    /// Signals that can be fired on this object
    fn supported_signals(&self) -> Signals;
}

#[macro_export]
macro_rules! kernel_object {
    ($class:ty, $signals:expr) => {
        impl $crate::object::KernelObject for $class {
            fn as_any(&self) -> &dyn core::any::Any {
                self
            }

            fn supported_signals(&self) -> rtl::signal::Signals {
                $signals
            }
        }

        impl core::ops::Deref for $class {
            type Target = $crate::object::KernelObjectBase;

            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }

        unsafe impl Send for $class {}
        unsafe impl Sync for $class {}
    };
}
