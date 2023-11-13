use core::sync::atomic::AtomicUsize;
use qrwlock::RwLock;

pub struct RefRwlock<T> {
    data: RwLock<T>,
    cnt: AtomicUsize,
}

impl<T> RefRwlock<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: RwLock::new(data),
            cnt: AtomicUsize::new(0),
        }
    }
}
