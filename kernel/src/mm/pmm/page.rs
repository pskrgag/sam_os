use crate::mm::vmm::layout::vmm_range;
use core::ptr::NonNull;
use hal::address::{Pfn, VirtualAddress};
use hal::page::Page as HalPage;

/// Owned page
pub struct Page(pub(crate) Pfn);

impl Page {
    pub fn pfn(&self) -> Pfn {
        self.0
    }
}

pub fn page_array_base() -> NonNull<HalPage> {
    let range = vmm_range(loader_protocol::VmmLayoutKind::PageArray);

    unsafe { NonNull::new_unchecked(range.start().to_raw_mut()) }
}
