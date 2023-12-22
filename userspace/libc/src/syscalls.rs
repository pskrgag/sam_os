#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

use shared::syscalls::SyscallList;
use shared::vmm::MappingType;

pub fn write(data: &str) -> usize {
    unsafe { syscall2(SyscallList::SYS_WRITE, data.as_ptr() as usize, data.len()) }
}

pub fn allocate(size: usize, tp: MappingType) -> usize {
    unsafe { syscall2(SyscallList::SYS_VM_ALLOCATE, size, tp.into()) }
}
