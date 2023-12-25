#![macro_use]

use core::{
    fmt::Debug,
    mem::size_of,
    ops::{Add, BitAnd, Not, Shl, Shr, Sub},
};

use rtl::arch::{PAGE_SIZE, PAGE_SHIFT};

extern "C" {
    static load_addr: usize;
    static start: usize;
    static mmio_end: usize;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        #[allow(unused_unsafe)]
        unsafe {
            &$a as *const usize as usize
        }
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

#[inline]
pub fn num_pages(size: usize) -> usize {
    size.next_multiple_of(PAGE_SIZE) >> PAGE_SHIFT
}

#[inline]
pub fn genmask<T>(h: T, l: T) -> T
where
    T: Shl<Output = T>
        + Add<Output = T>
        + Shr<Output = T>
        + Sub<Output = T>
        + Not<Output = T>
        + BitAnd<Output = T>
        + TryFrom<usize>
        + Copy,
    <T as TryFrom<usize>>::Error: Debug,
{
    let bits_per_t: T = T::try_from(size_of::<T>() * 8).unwrap();
    let one: T = T::try_from(1_usize).unwrap();
    let zero: T = T::try_from(0_usize).unwrap();

    (!zero - (one << l) + one) & (!zero >> (bits_per_t - one - h))
}

pub fn ref_to_usize<T>(rf: &T) -> usize {
    rf as *const _ as usize
}

pub fn ref_mut_to_usize<T>(rf: &mut T) -> usize {
    rf as *mut _ as usize
}
