use core::arch::asm;
use rtl::error::ErrorType;

#[inline]
pub unsafe fn syscall(args: [usize; 8]) -> Result<usize, ErrorType> {
    let mut ret: isize;

    asm!(
        "svc #0",
        inlateout("x0") args[0] => ret,
        in("x1") args[1],
        in("x2") args[2],
        in("x3") args[3],
        in("x4") args[4],
        in("x5") args[5],
        in("x6") args[6],
        in("x7") args[7],
        options(nostack, preserves_flags)
    );

    if ret < 0 {
        Err(ErrorType::from_bits((-ret) as usize).unwrap())
    } else {
        Ok(ret as usize)
    }
}
