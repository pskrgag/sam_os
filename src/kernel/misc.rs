#![macro_use]

use crate::arch;
use core::{
    ops::{Add, Shl, Shr, Sub, BitAnd, Not},
    mem::size_of,
    fmt::Debug,
};

extern "C" {
    static load_addr: usize;
    static start: usize;
    static mmio_end: usize;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        unsafe { &$a as *const usize as usize }
    }};
}

pub const _1GB: usize = 1 << 30;
pub const _2MB: usize = 2 << 20;
pub const _4KB: usize = 1 << 12;

#[inline]
pub fn kernel_offset() -> usize {
    linker_var!(start) - linker_var!(load_addr)
}

#[inline]
pub fn image_size() -> usize {
    linker_var!(mmio_end) - linker_var!(start)
}

pub fn num_pages(size: usize) -> usize {
    size.next_multiple_of(arch::PAGE_SIZE) >> arch::PAGE_SHIFT
}

#[inline]
pub fn genmask<T>(h: T, l: T) -> T
where
    T: Shl<Output = T> + Add<Output = T> + Shr<Output = T> + Sub<Output = T> + Not<Output = T> + BitAnd<Output = T> + TryFrom<usize> + Copy,
    <T as TryFrom<usize>>::Error: Debug
{
    let bits_per_t: T = T::try_from(size_of::<T>() * 8).unwrap();
    let one: T = T::try_from(1_usize).unwrap();
    let zero: T = T::try_from(0_usize).unwrap();

    (!zero - (one << l) + one) & (!zero >> (bits_per_t - one - h))
}
