use crate::syscalls::Syscall;
use shared::vmm::alloc::BackendAllocator;
use shared::arch::PAGE_SIZE;
use shared::vmm::MappingType;

pub struct SyscallBackend;
pub const SyscallBackendImpl: SyscallBackend = SyscallBackend{};

unsafe impl Sync for SyscallBackend { }

impl BackendAllocator for SyscallBackend {
    fn allocate(&self, num_pages: usize) -> Option<*mut u8> {
        Syscall::vm_allocate(num_pages * PAGE_SIZE, MappingType::USER_DATA).ok()
    }

    fn free(&self, p: *const u8, num_pages: usize) -> *mut u8 {
        todo!()
    }
}
