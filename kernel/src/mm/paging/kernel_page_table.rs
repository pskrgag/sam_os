use crate::{
    kernel::locking::spinlock::{Spinlock, SpinlockGuard},
    mm::paging::page_table::PageTable,
};

pub static KERNEL_PAGE_TABLE: Spinlock<PageTable> = Spinlock::new(PageTable::default(true));

pub fn init() {
    let mut table = KERNEL_PAGE_TABLE.lock();
    *table = PageTable::new(true).expect("Failed to allocate tt base");

    println!("Allocated kernel page table base");
}

pub fn kernel_page_table() -> SpinlockGuard<'static, PageTable> {
    KERNEL_PAGE_TABLE.lock()
}
