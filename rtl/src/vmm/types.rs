use crate::arch;
use core::ops::Add;
use core::{
    fmt::{self, Debug},
    ops::Sub,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
#[repr(transparent)]
pub struct PhysAddr(usize);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
#[repr(transparent)]
pub struct VirtAddr(usize);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
#[repr(transparent)]
pub struct Pfn(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct MemRange<T: Address + core::fmt::Debug> {
    pub start: T,
    pub size: usize,
}

impl<T: Address + Debug> core::fmt::Debug for MemRange<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "MemRange [ start: {:x}, size: {:x} ]",
            self.start.bits(),
            self.size
        ))
    }
}

pub trait Address {
    fn bits(&self) -> usize;
    fn set_bits(&mut self, bits: usize);

    fn is_null(&self) -> bool {
        self.bits() == 0
    }

    fn round_up(&mut self, to: usize) -> &mut Self {
        assert!(to.is_power_of_two());

        let round_mask = to - 1;
        let rounded = ((self.bits() - 1) | round_mask) + 1;

        self.set_bits(rounded);
        self
    }

    fn add(&mut self, add: usize) -> &mut Self {
        self.set_bits(self.bits() + add);
        self
    }

    #[inline]
    fn is_aligned(&self, order: usize) -> bool {
        self.bits() & ((1 << order) - 1) == 0
    }

    #[inline]
    fn is_page_aligned(&self) -> bool {
        self.is_aligned(arch::PAGE_SHIFT)
    }

    #[inline]
    fn round_down_page(&mut self) -> &mut Self {
        self.set_bits(self.bits() & !((1_usize << arch::PAGE_SHIFT) - 1));
        self
    }

    #[inline]
    fn round_up_page(&mut self) -> &mut Self {
        self.round_up(arch::PAGE_SIZE)
    }

    fn page_offset(&self) -> usize {
        self.bits() & ((1_usize << arch::PAGE_SHIFT) - 1)
    }
}

impl<T: Copy + Address + From<usize> + Ord + core::fmt::Debug> MemRange<T> {
    pub const fn new(start: T, size: usize) -> Self {
        Self { start, size }
    }

    pub const fn start(&self) -> T {
        self.start
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub fn align_page(&mut self) {
        self.size += self.start.bits() & (arch::PAGE_SIZE - 1);
        self.start
            .set_bits(self.start().bits() & !(arch::PAGE_SIZE - 1));

        self.size.round_up(arch::PAGE_SIZE);
    }

    pub fn truncate(&mut self, size: usize) -> bool {
        self.start.add(size);

        if !self.size.overflowing_sub(size).1 {
            self.size -= size;
            true
        } else {
            false
        }
    }

    pub fn end(&self) -> T {
        T::from(self.start.bits() + self.size)
    }

    pub fn contains_addr(&self, addr: T) -> bool {
        self.start <= addr && self.start.bits() + self.size > addr.bits()
    }

    pub fn contains_range(&self, other: MemRange<VirtAddr>) -> bool {
        self.start().bits() <= other.start().bits()
            && other.start().bits() + other.size() < self.start().bits() + self.size()
    }
}

impl From<usize> for PhysAddr {
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl From<PhysAddr> for usize {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl From<Pfn> for usize {
    fn from(addr: Pfn) -> Self {
        addr.0
    }
}

impl From<PhysAddr> for Pfn {
    fn from(addr: PhysAddr) -> Self {
        Pfn::from(addr.0 >> arch::PAGE_SHIFT)
    }
}

impl From<Pfn> for PhysAddr {
    fn from(addr: Pfn) -> Self {
        PhysAddr::from(addr.0 << arch::PAGE_SHIFT)
    }
}

impl From<VirtAddr> for usize {
    fn from(addr: VirtAddr) -> Self {
        addr.0
    }
}

impl From<usize> for Pfn {
    fn from(addr: usize) -> Self {
        Pfn(addr)
    }
}

impl From<usize> for VirtAddr {
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl Add for PhysAddr {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for PhysAddr {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Sub for Pfn {
    type Output = usize;

    fn sub(self, other: Self) -> Self::Output {
        self.0 - other.0
    }
}

impl Sub for VirtAddr {
    type Output = usize;

    fn sub(self, other: Self) -> Self::Output {
        self.0 - other.0
    }
}

impl Add for VirtAddr {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Add<usize> for VirtAddr {
    type Output = usize;

    fn add(self, other: usize) -> Self::Output {
        self.0 + other
    }
}

impl PhysAddr {
    // We need get() to be const in some cases.
    // so we can't remove it
    pub const fn get(&self) -> usize {
        self.0
    }

    pub const fn to_pfn(&self) -> usize {
        self.0 >> arch::PAGE_SHIFT
    }

    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }
}

impl Pfn {
    pub const fn get(&self) -> usize {
        self.0
    }

    pub const fn new(pfn: usize) -> Self {
        Self(pfn)
    }
}

impl VirtAddr {
    pub const fn new(ptr: usize) -> Self {
        Self(ptr)
    }

    pub fn from_raw<T>(ptr: *const T) -> Self {
        Self(ptr as usize)
    }

    pub fn to_raw<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn to_raw_mut<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// # Safety
    ///
    /// Caller should be sure that pointer points to [`[count; T]`]
    pub unsafe fn as_slice_mut<T>(&mut self, count: usize) -> &mut [T] {
        core::slice::from_raw_parts_mut(self.0 as *mut T, count)
    }

    /// # Safety
    ///
    /// Caller should be sure that pointer points to [`[count; T]`]
    pub unsafe fn as_slice_at_offset_mut<T>(&mut self, count: usize, offset: usize) -> &mut [T] {
        core::slice::from_raw_parts_mut((self.0 + offset) as *mut T, count)
    }
}

impl Address for VirtAddr {
    #[inline]
    fn bits(&self) -> usize {
        self.0
    }

    #[inline]
    fn set_bits(&mut self, bits: usize) {
        self.0 = bits;
    }
}

impl Address for usize {
    #[inline]
    fn bits(&self) -> usize {
        *self
    }

    #[inline]
    fn set_bits(&mut self, bits: usize) {
        *self = bits;
    }
}
impl Address for PhysAddr {
    #[inline]
    fn bits(&self) -> usize {
        self.0
    }

    #[inline]
    fn set_bits(&mut self, bits: usize) {
        self.0 = bits;
    }
}

impl<T> From<*const T> for VirtAddr {
    fn from(addr: *const T) -> Self {
        Self(addr as usize)
    }
}

impl<T> From<*mut T> for VirtAddr {
    fn from(addr: *mut T) -> Self {
        Self(addr as usize)
    }
}

#[cfg(feature = "kernel")]
impl From<PhysAddr> for VirtAddr {
    fn from(addr: PhysAddr) -> Self {
        Self(addr.get() + arch::PHYS_OFFSET)
    }
}

#[cfg(feature = "kernel")]
impl From<VirtAddr> for PhysAddr {
    fn from(addr: VirtAddr) -> Self {
        Self(addr.bits() - arch::PHYS_OFFSET)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.0;

        fmt::LowerHex::fmt(&val, f)
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.0;

        fmt::LowerHex::fmt(&val, f)
    }
}
