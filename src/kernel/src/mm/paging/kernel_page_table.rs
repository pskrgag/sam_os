use crate::{
    kernel::locking::spinlock::{Spinlock, SpinlockGuard},
    mm::paging::page_table::PageTable,
};

pub static KERNEL_PAGE_TABLE: Spinlock<PageTable> = Spinlock::new(PageTable::default());

#[unsafe(no_mangle)]
pub static mut PAGE_TABLE_BASE: usize = 0;

pub fn init() {
    let mut table = KERNEL_PAGE_TABLE.lock();
    *table = PageTable::new().expect("Failed to allocate tt base");

    println!("Allocated kernel page table base");

    unsafe {
        PAGE_TABLE_BASE = table.base().get();
    }
}

pub fn kernel_page_table() -> SpinlockGuard<'static, PageTable> {
    KERNEL_PAGE_TABLE.lock()
}
