use core::sync::atomic::AtomicUsize;

#[repr(C)]
pub struct Page {
    refcount: AtomicUsize,
}
