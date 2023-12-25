#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

use rtl::syscalls::SyscallList;
use rtl::vmm::MappingType;
use rtl::error::ErrorType;

pub enum Syscall<'a> {
    Write(&'a str, usize),
    VmAllocate(usize, MappingType),
}

impl<'a> Syscall<'a> {
   pub fn vm_allocate(size: usize, mt: MappingType) -> Result<*mut u8, ErrorType> {
       unsafe { Ok(syscall(Self::VmAllocate(size, mt).as_args())? as *mut u8) }
   }

   pub fn debug_write(s: &'a str) -> Result<(), ErrorType> {
       unsafe { syscall(Self::Write(s, s.len()).as_args())? };
       Ok(())
   }

   pub fn as_args(self) -> [usize; 8] {
        match self {
            Syscall::Write(string, size) => [
                SyscallList::SYS_WRITE.into(),
                string.as_ptr() as *const u8 as usize,
                size,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmAllocate(size, tp) => [
                SyscallList::SYS_VM_ALLOCATE.into(),
                size,
                tp.into(),
                0,
                0,
                0,
                0,
                0,
            ]
        }
   }
}
