use core::{
    fmt::Debug,
    mem::size_of,
    ops::{Add, BitAnd, Not, Shl, Shr, Sub},
};

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

/// # Safety
///
/// [`v`] should be created by ref_mut_to_usize of ref_to_usize
pub unsafe fn usize_to_ref<T>(v: usize) -> &'static T {
    &*(v as *const u8 as *const T)
}

#[repr(C)]
pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
}

#[macro_export]
macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:expr) => {{
        // const block expression to encapsulate the static

        static ALIGNED: &rtl::misc::AlignedAs<$align_ty, [u8]> = &rtl::misc::AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        &raw const $a as usize
    }};
}
