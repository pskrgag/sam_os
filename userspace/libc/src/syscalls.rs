#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

use rtl::error::ErrorType;
use rtl::handle::Handle;
use rtl::syscalls::SyscallList;
use rtl::vmm::types::VirtAddr;
use rtl::vmm::MappingType;

pub enum Syscall<'a> {
    Write(&'a str),
    VmAllocate(usize, MappingType),
    VmObjectCreate(&'a [u8], MappingType, VirtAddr),
    TaskCreateFromVmo(&'a str, &'a [Handle], VirtAddr),
    TaskStart(Handle),
}

impl<'a> Syscall<'a> {
    pub fn task_create_from_vmo(
        name: &'a str,
        vmos: &'a [Handle],
        ep: VirtAddr,
    ) -> Result<Handle, ErrorType> {
        unsafe { Ok(syscall(Self::TaskCreateFromVmo(name, vmos, ep).as_args())? as Handle) }
    }

    pub fn vm_allocate(size: usize, mt: MappingType) -> Result<*mut u8, ErrorType> {
        unsafe { Ok(syscall(Self::VmAllocate(size, mt).as_args())? as *mut u8) }
    }

    pub fn debug_write(s: &'a str) -> Result<(), ErrorType> {
        unsafe { syscall(Self::Write(s).as_args())? };
        Ok(())
    }

    pub fn task_start(h: Handle) -> Result<(), ErrorType> {
        unsafe { syscall(Self::TaskStart(h).as_args())? };
        Ok(())
    }

    pub fn create_vm_object(
        b: &'a [u8],
        tp: MappingType,
        load_addr: VirtAddr,
    ) -> Result<Handle, ErrorType> {
        unsafe { Ok(syscall(Self::VmObjectCreate(b, tp, load_addr).as_args())? as Handle) }
    }

    pub fn as_args(self) -> [usize; 8] {
        match self {
            Syscall::Write(string) => [
                SyscallList::SYS_WRITE.into(),
                string.as_ptr() as *const u8 as usize,
                string.len(),
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
            ],
            Syscall::VmObjectCreate(buf, tp, load_addr) => [
                SyscallList::SYS_VM_CREATE_VM_OBJECT.into(),
                buf.as_ptr() as usize,
                buf.len(),
                tp.into(),
                load_addr.into(),
                0,
                0,
                0,
            ],
            Syscall::TaskCreateFromVmo(name, handles, ep) => [
                SyscallList::SYS_TASK_CREATE_FROM_VMO.into(),
                name.as_ptr() as usize,
                name.len(),
                handles.as_ptr() as usize,
                handles.len(),
                ep.into(),
                0,
                0,
            ],
            Syscall::TaskStart(handle) => [
                SyscallList::SYS_TASK_START.into(),
                handle,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        }
    }
}
