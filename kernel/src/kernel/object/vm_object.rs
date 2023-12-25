use crate::kernel::locking::spinlock::Spinlock;
use crate::kernel::misc::num_pages;
use crate::mm::allocators::page_alloc::page_allocator;
use crate::mm::paging::kernel_page_table::kernel_page_table;
use object_lib::object;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;
use rtl::arch::PAGE_SIZE;

struct VmObjectInner {
    start: PhysAddr,
    pages: usize,
}

#[derive(object)]
pub struct VmObject {
    inner: Arc<Spinlock<VmObjectInner>>,
}

impl VmObjectInner {
    pub fn from_buffer(b: &[u8]) -> Option<Self> {
        let pages = num_pages(b.len());

        let p: PhysAddr = page_allocator().alloc(pages)?.into();

        let va = kernel_page_table()
            .map(
                None,
                MemRange::new(VirtAddr::from(p), pages * PAGE_SIZE),
                MappingType::KERNEL_DATA,
            )
            .ok()?;

        todo!()
    }
}
