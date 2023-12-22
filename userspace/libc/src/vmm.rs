use crate::syscalls::allocate;
use shared::vmm::MappingType;

pub fn vm_allocate(size: usize, tp: MappingType) -> *mut u8 {
    allocate(size, tp.into()) as *mut u8
}
