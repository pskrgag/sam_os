use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{RawWaker, RawWakerVTable, Waker as CoreWaker};

use alloc::sync::Arc;

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop_waker);

unsafe fn clone(data: *const ()) -> RawWaker {
    unsafe {
        let arc = Arc::from_raw(data as *const Waker).clone();

        core::mem::forget(arc);
        RawWaker::new(data, &VTABLE)
    }
}

unsafe fn wake(data: *const ()) {
    unsafe {
        let arc = Arc::from_raw(data as *const Waker);
        arc.wake()
    }
}

unsafe fn wake_by_ref(data: *const ()) {
    unsafe { wake(data) }
}

unsafe fn drop_waker(data: *const ()) {
    unsafe { drop(Arc::from_raw(data as *const Waker)) }
}

pub struct Waker<'a> {
    bit: &'a AtomicU64,
    index: u8,
}

impl Waker<'_> {
    fn wake(&self) {
        self.bit.fetch_or(1u64 << self.index, Ordering::Relaxed);
    }
}

pub struct WakerPage {
    notified: AtomicU64,
}

impl WakerPage {
    pub fn new() -> Self {
        Self {
            notified: AtomicU64::new(0),
        }
    }

    pub fn initialize(&self, task: u8) {
        // Newly added task is ready to be polled
        self.notify(task);
    }

    pub fn notify(&self, task: u8) {
        self.notified.fetch_or(1 << task, Ordering::Relaxed);
    }

    pub fn num_entries() -> usize {
        64
    }

    pub fn notified(&self) -> impl Iterator<Item = u8> {
        let mask = self.notified.load(Ordering::Acquire);
        let mut bit = 0;

        core::iter::from_fn(move || {
            while bit < Self::num_entries() && mask & (1u64 << bit) == 0 {
                bit += 1;
            }

            if bit < Self::num_entries() {
                let res = bit;

                self.notified.fetch_and(!(1 << res), Ordering::Relaxed);
                bit += 1;
                Some(res as u8)
            } else {
                None
            }
        })
    }

    pub fn waker(&self, index: u8) -> CoreWaker {
        let arc = Arc::new(Waker {
            bit: &self.notified,
            index,
        });
        let raw = RawWaker::new(Arc::into_raw(arc) as *const _, &VTABLE);

        unsafe { CoreWaker::from_raw(raw) }
    }
}
