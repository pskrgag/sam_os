use core::arch::asm;
use rtl::error::ErrorType;

#[inline]
pub unsafe fn syscall(_args: [usize; 8]) -> Result<usize, ErrorType> {
    unimplemented!();
}

