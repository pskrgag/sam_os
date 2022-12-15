use alloc::boxed::Box;
use core::arch::asm;

pub unsafe fn kernel_thread_entry_point<T>() {
    let addr: *mut T;

    asm!("mov   {}, x19", out(reg) addr);
    asm!("mov   x0, x19");
    asm!("br    x20");

    drop(Box::from_raw(addr));
}

pub fn idle_thread(_: ()) -> Option<()> {
    loop {}
}
