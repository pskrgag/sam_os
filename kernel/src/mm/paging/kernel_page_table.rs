use crate::{
    kernel::locking::spinlock::{Spinlock, SpinlockGuard},
    mm::paging::page_table::PageTable,
};

use spin::once::Once;

pub static KERNEL_PAGE_TABLE: Once<Spinlock<PageTable>> = Once::new();

#[no_mangle]
pub static mut PAGE_TABLE_BASE: usize = 0;

pub fn init() {
    KERNEL_PAGE_TABLE.call_once(|| Spinlock::new(PageTable::new()));

    println!("Allocated kernel page table base");

    unsafe {
        PAGE_TABLE_BASE = KERNEL_PAGE_TABLE.get_unchecked().lock().base().get();
    }
}

pub fn kernel_page_table() -> SpinlockGuard<'static, PageTable> {
    KERNEL_PAGE_TABLE.get().expect("Must be already initialized").lock()
}
