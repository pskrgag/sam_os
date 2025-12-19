use crate::{
    kernel::locking::spinlock::{Spinlock, SpinlockGuard},
    mm::paging::page_table::PageTable,
};
use spin::once::Once;

pub static KERNEL_PAGE_TABLE: Once<Spinlock<PageTable>> = Once::new();

#[unsafe(no_mangle)]
pub static mut PAGE_TABLE_BASE: usize = 0;

pub fn init(arg: &loader_protocol::LoaderArg) {
    KERNEL_PAGE_TABLE.call_once(|| unsafe { Spinlock::new(PageTable::from(arg.tt_base.into())) });
}

pub fn kernel_page_table() -> SpinlockGuard<'static, PageTable> {
    unsafe { KERNEL_PAGE_TABLE.get_unchecked().lock() }
}
