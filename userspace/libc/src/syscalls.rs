#![allow(dead_code)]

use core::arch::asm;

#[inline]
unsafe fn syscall0(n: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        in("x0") n,
        lateout("x1") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        in("x0") n as usize,
        inlateout("x1") arg1 => ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        in("x0") n,
        inlateout("x1") arg1 => ret,
        in("x2") arg2,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        in("x8") n,
        inlateout("x0") arg1 => ret,
        in("x1") arg2,
        in("x2") arg3,
        options(nostack, preserves_flags)
    );
    ret
}

pub fn nop() {
    unsafe { syscall0(10) };
}

pub fn write(data: &str) -> usize {
    unsafe { syscall2(0, data.as_ptr() as usize, data.len()) }
}
