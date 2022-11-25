use crate::arch;
use core::ops::Add;
use core::{fmt, ops::Sub};

#[derive(Clone, Copy)]
pub struct PhysAddr(usize);

#[derive(Clone, Copy)]
pub struct VirtAddr(usize);

#[derive(Clone, Copy)]
pub struct Pfn(usize);

pub struct MemRange<T> {
    start: T,
    size: usize,
}

impl<T: Copy> MemRange<T> {
    pub const fn new(start: T, size: usize) -> Self {
        Self {
            start: start,
            size: size,
        }
    }

    pub const fn start(&self) -> T {
        self.start
    }

    pub const fn size(&self) -> usize {
        self.size
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
        Self(addr.get() - arch::PHYS_OFFSET)
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

impl PhysAddr {
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

    pub const fn get(&self) -> usize {
        self.0
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

    pub fn add(&mut self, add: usize) {
        self.0 += add;
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
