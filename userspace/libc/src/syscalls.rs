#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

use rtl::error::ErrorType;
use rtl::handle::Handle;
use rtl::syscalls::SyscallList;

pub enum Syscall<'a> {
    Write(&'a str),
    Invoke(Handle, usize, &'a [usize]),
    Yield,
}

impl<'a> Syscall<'a> {
    pub fn debug_write(s: &'a str) -> Result<(), ErrorType> {
        unsafe { syscall(Self::Write(s).as_args())? };
        Ok(())
    }

    pub fn invoke(h: Handle, op: usize, args: &'a [usize]) -> Result<usize, ErrorType> {
        unsafe { syscall(Self::Invoke(h, op, args).as_args()) }
    }

    pub fn sys_yield() {
        unsafe { syscall(Self::Yield.as_args()).unwrap() };
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
            Syscall::Yield => [
                SyscallList::SYS_YIELD.into(),
                0,
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
