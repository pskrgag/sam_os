use crate::arch::{self, PAGE_SIZE};
use core::ops::Add;
use core::{fmt, ops::Sub};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct PhysAddr(usize);

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct VirtAddr(usize);

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Pfn(usize);

#[derive(Clone, Copy)]
pub struct MemRange<T: Address> {
    start: T,
    size: usize,
}

pub trait Address {
    fn bits(&self) -> usize;
    fn set_bits(&mut self, bits: usize);

    fn round_up(&mut self, to: usize) -> &mut Self {
        assert!(to.is_power_of_two());

        let round_mask = to - 1;
        let rounded = (self.bits() | round_mask) + 1;

        self.set_bits(rounded);
        self
    }

    fn add(&mut self, add: usize) -> &mut Self {
        self.set_bits(self.bits() + add);
        self
    }

    #[inline]
    fn is_page_aligned(&self) -> bool {
        self.bits() & ((1 << arch::PAGE_SHIFT) - 1) == 0
    }

    #[inline]
    fn round_down_page(&mut self) -> &mut Self {
        self.set_bits(self.bits() & !((1_usize << arch::PAGE_SHIFT) - 1));
        self
    }

    fn page_offset(&self) -> usize {
        self.bits() & ((1_usize << arch::PAGE_SHIFT) - 1)
    }
}

impl<T: Copy + Address + From<usize>> MemRange<T> {
    pub const fn new(start: T, size: usize) -> Self {
        Self {
            start,
            size,
        }
    }

    pub const fn start(&self) -> T {
        self.start
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub fn truncate(&mut self, size: usize) -> bool {
        self.start.add(size);
        self.size.overflowing_sub(size).1
    }

    pub fn max_user() -> Self {
        Self::new(T::from(PAGE_SIZE), (1 << 39) - PAGE_SIZE)
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

impl From<PhysAddr> for VirtAddr {
    fn from(addr: PhysAddr) -> Self {
        Self(addr.get() + arch::PHYS_OFFSET)
    }
}

impl From<VirtAddr> for PhysAddr {
    fn from(addr: VirtAddr) -> Self {
        Self(addr.bits() - arch::PHYS_OFFSET)
    }
}

impl Add for PhysAddr {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
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
