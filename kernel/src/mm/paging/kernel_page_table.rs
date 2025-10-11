use crate::{
    kernel::locking::spinlock::{Spinlock, SpinlockGuard},
    mm::paging::page_table::PageTable,
};

pub static KERNEL_PAGE_TABLE: Spinlock<PageTable> = Spinlock::new(PageTable::default());

#[unsafe(no_mangle)]
pub static mut PAGE_TABLE_BASE: usize = 0;

pub fn init(arg: &loader_protocol::LoaderArg) {
    let mut table = KERNEL_PAGE_TABLE.lock();

    unsafe { *table = PageTable::from(arg.tt_base.into()) };
}

pub fn kernel_page_table() -> SpinlockGuard<'static, PageTable> {
    KERNEL_PAGE_TABLE.lock()
}
