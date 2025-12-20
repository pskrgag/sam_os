use crate::tasks::thread::Thread;
use crate::sync::Mutex;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::any::Any;
use signals::Signals;

pub mod capabilities;
pub mod handle;
pub mod handle_table;

pub mod factory_object;
pub mod port_object;
pub mod signals;

pub type ObserverHandler = Box<dyn Fn(signals::Signals) -> bool + Send>;

#[derive(Default)]
struct KernelObjectBaseInner {
    signals: signals::Signals,
    observers: Vec<(ObserverHandler, Weak<Thread>)>,
}

pub struct KernelObjectBase(Mutex<KernelObjectBaseInner>);

impl KernelObjectBase {
    pub fn new() -> Self {
        Self(Mutex::new(KernelObjectBaseInner::default()))
    }

    pub fn add_observer(&self, obs: ObserverHandler) {
        use crate::sched::current;

        let mut inner = self.0.lock();

        if !obs(inner.signals) {
            let cur = current();

            inner.observers.push((obs, Arc::downgrade(&cur)));
            // cur.sleep(thread_object::ThreadSleepReason::Event);
        }
    }

    pub fn signal(&self, sig: Signals) {
        let mut inner = self.0.lock();

        inner.signals |= sig;

        let signals = inner.signals;

        inner.observers.retain(|x| {
            let res = x.0(signals);

            if res {
                if let Some(sleeper) = x.1.upgrade() {
                    sleeper.wake();
                }
            }

            !res
        });
    }
}

// All exposed kernel objects must be derived from this trait
pub trait KernelObject: Send {
    /// Expose yourself as Any to allow storing in capability table
    fn as_any(&self) -> &dyn Any;

    /// Signal an event
    fn signal(&self, signals: Signals);

    /// Signal an event
    fn wait_event(&self, obs: ObserverHandler);
}
