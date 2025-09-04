use crate::vmm::vms::vms;
use rtl::arch::PAGE_SIZE;
use rtl::vmm::alloc::BackendAllocator;
use rtl::vmm::MappingType;

pub struct SyscallBackend;

#[allow(non_upper_case_globals)]
pub const SyscallBackendImpl: SyscallBackend = SyscallBackend {};

unsafe impl Sync for SyscallBackend {}

impl BackendAllocator for SyscallBackend {
    fn allocate(&self, num_pages: usize) -> Option<*mut u8> {
        vms()
            .vm_allocate(num_pages * PAGE_SIZE, MappingType::USER_DATA)
            .ok()
    }

    fn free(&self, _p: *const u8, _num_pages: usize) {
        todo!()
    }
}
