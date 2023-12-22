#![allow(dead_code)]

use core::arch::asm;

#[inline]
unsafe fn syscall0(n: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        in("x0") n,
        lateout("x0") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        inlateout("x0") n as usize => ret,
        in("x1") arg1,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline]
unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret: usize;
    asm!(
        "svc #0",
        inlateout("x0") n => ret,
        in("x1") arg1,
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
        inlateout("x0") n => ret,
        in("x1") arg1,
        in("x2") arg2,
        in("x3") arg3,
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

pub fn allocate(data: &str) -> usize {
    unsafe { syscall2(1, data.as_ptr() as usize, data.len()) }
}
