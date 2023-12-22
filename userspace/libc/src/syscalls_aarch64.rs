use core::arch::asm;
use shared::syscalls::*;

#[inline]
pub unsafe fn syscall0(n: SyscallList) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        inlateout("x0") <SyscallList as Into<usize>>::into(n) => ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
pub unsafe fn syscall1(n: SyscallList, arg1: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        inlateout("x0") <SyscallList as Into<usize>>::into(n) => ret,
        in("x1") arg1,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
pub unsafe fn syscall2(n: SyscallList, arg1: usize, arg2: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        inlateout("x0") <SyscallList as Into<usize>>::into(n) => ret,
        in("x1") arg1,
        in("x2") arg2,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
pub unsafe fn syscall3(n: SyscallList, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        inlateout("x0") <SyscallList as Into<usize>>::into(n) => ret,
        in("x1") arg1,
        in("x2") arg2,
        in("x3") arg3,
        options(nostack, preserves_flags)
    );
    ret
}
