use super::MemRange;
use super::layout::KERNEL_LAYOUT;
use super::page_table::{PageKind, PagePerms, PageTable};
use super::regions::whole_ram;
use core::mem::size_of;
use hal::address::VirtAddr;
use hal::arch::PAGE_SIZE;
use hal::page::Page;
use loader_protocol::VmmLayoutKind;

pub fn init(tt: &mut PageTable) {
    let ram = whole_ram();
    let pages_to_alloc = (size_of::<Page>() * ram.count).next_multiple_of(PAGE_SIZE) / PAGE_SIZE;

    let regions = &KERNEL_LAYOUT[VmmLayoutKind::PageArray as usize];
    let pages =
        super::alloc::alloc_pages(pages_to_alloc).expect("Failed to allocate pages for page array");

    assert!(regions.size >= pages_to_alloc * PAGE_SIZE);

    let va_range = MemRange::new(VirtAddr::new(regions.base), pages_to_alloc * PAGE_SIZE);
    let pa_range = MemRange::new(pages, pages_to_alloc * PAGE_SIZE);

    tt.map_pages(va_range, pa_range, PagePerms::ReadWrite, PageKind::Normal);
}
