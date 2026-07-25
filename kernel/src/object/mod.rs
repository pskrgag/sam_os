use crate::sync::Spinlock;
use adt::Vec;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::any::Any;
use core::future::Future;
use core::ops::Deref;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use core::task::{Context, Poll};
use rtl::error::ErrorType;
use rtl::signal::Signals;

pub mod capabilities;
pub mod handle;
pub mod handle_table;

pub mod factory_object;
pub mod port_object;

/// Callback that is called on event
struct Observer {
    callback: Box<dyn Fn(Signals) -> bool + Send>,
    token: ObserverToken,
}

/// Unique token to remove observer
#[derive(PartialEq, Eq, Copy, Clone)]
struct ObserverToken(u64);

/// Kernel object base
pub struct KernelObjectBase(Spinlock<KernelObjectBaseInner>);

#[derive(Default)]
struct KernelObjectBaseInner {
    signals: Signals,
    observers: Vec<Observer>,
}

impl Observer {
    fn new(callback: Box<dyn Fn(Signals) -> bool + Send>) -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);

        let id = ID.fetch_add(1, Ordering::Relaxed);

        // TODO: this is bold assumption, but let's keep it for now.
        assert!(id != u64::MAX);

        Self {
            callback,
            token: ObserverToken(id),
        }
    }

    fn token(&self) -> ObserverToken {
        self.token
    }
}

impl KernelObjectBaseInner {
    fn add_observer(&mut self, obs: Observer) -> Result<(), ErrorType> {
        if !(obs.callback)(self.signals) {
            self.observers.try_push(obs)
        } else {
            Ok(())
        }
    }
}

impl KernelObjectBase {
    pub fn new() -> Self {
        Self(Spinlock::new(KernelObjectBaseInner::default()))
    }

    pub fn signals(&self) -> Signals {
        self.0.lock().signals
    }

    fn remove_observer(&self, token: ObserverToken) {
        let mut inner = self.0.lock();

        inner.observers.retain(|x| x.token() != token);
    }

    fn add_observer(&self, obs: Observer) -> Result<ObserverToken, ErrorType> {
        let token = obs.token();

        self.0.lock().add_observer(obs)?;
        Ok(token)
    }

    pub async fn wait_signal(&self, sig: Signals) -> Result<(), ErrorType> {
        struct Wait<'a> {
            sig: Signals,
            base: &'a KernelObjectBase,
            // TODO: add atomic signals
            pending: Arc<AtomicU8>,
            token: Option<ObserverToken>,
        }

        impl Drop for Wait<'_> {
            fn drop(&mut self) {
                if let Some(token) = self.token {
                    self.base.remove_observer(token);
                }
            }
        }

        impl Future for Wait<'_> {
            type Output = Result<(), ErrorType>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if self.pending.load(Ordering::Acquire) != 0 {
                    return Poll::Ready(Ok(()));
                }

                if self.token.is_none() {
                    let waker = cx.waker().clone();
                    let wait_sig = self.sig;
                    let pending = self.pending.clone();

                    let token = self.base.add_observer(Observer::new(
                        Box::try_new(move |sig: Signals| {
                            if sig.contains(wait_sig) {
                                pending.fetch_or(*(sig & wait_sig), Ordering::Release);
                                waker.wake_by_ref();
                                true
                            } else {
                                false
                            }
                        })
                        .map_err(|_| ErrorType::NoMemory)?,
                    ))?;
                    self.token = Some(token);
                }

                if self.pending.load(Ordering::Acquire) != 0 {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            }
        }

        let pending = Arc::try_new(AtomicU8::new(0)).map_err(|_| ErrorType::NoMemory)?;

        Wait {
            base: self,
            sig,
            pending,
            token: None,
        }
        .await
    }

    pub fn signal_clear(&self, sig: Signals) {
        let mut inner = self.0.lock();

        inner.signals &= !sig;
    }

    pub fn signal_fire(&self, sig: Signals) {
        let mut inner = self.0.lock();
        inner.signals |= sig;

        let signals = inner.signals;
        inner.observers.retain(|x| !(x.callback)(signals));
    }
}

pub struct WaitManyArg {
    pub obj: Arc<dyn KernelObject + Send>,
    pub waitfor: Signals,
    pub pending: Signals,
}

pub async fn wait_many(entries: &mut Vec<WaitManyArg>) -> Result<(), ErrorType> {
    struct Registration {
        obj: Arc<dyn KernelObject + Send>,
        pending: Arc<AtomicU8>,
        token: ObserverToken,
    }

    struct WaitMany<'a> {
        entries: &'a mut Vec<WaitManyArg>,
        registered: Vec<Registration>,
        polled: bool,
    }

    impl WaitMany<'_> {
        fn register_observers(&mut self, cx: &Context<'_>) -> Result<(), ErrorType> {
            for entry_index in 0..self.entries.len() {
                let obj = self.entries[entry_index].obj.clone();
                let waitfor = self.entries[entry_index].waitfor;
                let waker = cx.waker().clone();
                let pending = Arc::try_new(AtomicU8::new(0)).map_err(|_| ErrorType::NoMemory)?;

                // We need to clone to shut up rust....
                let callback_pending = pending.clone();

                // There we collect seen signals in call-back to fix lost wakeups. Collecting peding
                // signals after callback is too late.
                let token = obj.add_observer(Observer::new(
                    Box::try_new(move |sig: Signals| {
                        if sig.contains(waitfor) {
                            // release matches with release in observer.
                            callback_pending.fetch_or(*(sig & waitfor), Ordering::Release);
                            waker.wake_by_ref();

                            true
                        } else {
                            false
                        }
                    })
                    .map_err(|_| ErrorType::NoMemory)?,
                ))?;

                if let Err(err) = self.registered.try_push(Registration {
                    obj: obj.clone(),
                    pending,
                    token,
                }) {
                    obj.remove_observer(token);
                    return Err(err);
                }
            }

            Ok(())
        }

        fn collect_pending(&mut self) -> bool {
            let mut ready = false;
            let Self {
                entries,
                registered,
                ..
            } = self;

            for (entry, registration) in core::iter::zip(entries.iter_mut(), registered.iter()) {
                // acquire matches with release in observer.
                let pending = registration.pending.load(Ordering::Acquire);

                if pending != 0 {
                    entry.pending =
                        Signals::try_from(pending as usize).expect("Invalid pending signals");
                    ready = true;
                }
            }

            ready
        }
    }

    impl Drop for WaitMany<'_> {
        fn drop(&mut self) {
            for registration in &self.registered {
                registration.obj.remove_observer(registration.token);
            }
        }
    }

    impl Future for WaitMany<'_> {
        type Output = Result<(), ErrorType>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if !self.polled {
                if let Err(err) = self.register_observers(cx) {
                    return Poll::Ready(Err(err));
                }

                self.polled = true;
            }

            if self.collect_pending() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }
    }

    // Wait for any object to signal
    WaitMany {
        registered: Vec::with_capacity(entries.len())?,
        entries,
        polled: false,
    }
    .await?;

    Ok(())
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
