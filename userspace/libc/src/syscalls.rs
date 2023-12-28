#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

use rtl::error::ErrorType;
use rtl::handle::Handle;
use rtl::syscalls::SyscallList;
use rtl::vmm::types::VirtAddr;

pub enum Syscall<'a> {
    Write(&'a str),
    TaskCreateFromVmo(&'a str, &'a [Handle], VirtAddr),
    Invoke(Handle, usize, &'a [usize]),
}

impl<'a> Syscall<'a> {
    pub fn task_create_from_vmo(
        name: &'a str,
        vmos: &'a [Handle],
        ep: VirtAddr,
    ) -> Result<Handle, ErrorType> {
        unsafe { Ok(syscall(Self::TaskCreateFromVmo(name, vmos, ep).as_args())? as Handle) }
    }

    pub fn debug_write(s: &'a str) -> Result<(), ErrorType> {
        unsafe { syscall(Self::Write(s).as_args())? };
        Ok(())
    }

    pub fn invoke(h: Handle, op: usize, args: &'a [usize]) -> Result<usize, ErrorType> {
        unsafe { syscall(Self::Invoke(h, op, args).as_args()) }
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
            Syscall::Invoke(handle, op, args) => {
                let mut a = [0usize; 8];
                a[0] = SyscallList::SYS_INVOKE.into();
                a[1] = handle;
                a[2] = op;

                for i in 0..args.len() {
                    a[i + 3] = args[i];
                }

                a
            }
        }
    }
}
