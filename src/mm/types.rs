use crate::arch;
use core::{fmt, ops::Sub};

#[derive(Clone, Copy)]
pub struct PhysAddr(u64);

#[derive(Clone, Copy)]
pub struct VirtAddr(u64);

#[derive(Clone, Copy)]
pub struct Pfn(u64);

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
        Self(addr as u64)
    }
}

impl From<PhysAddr> for usize {
    fn from(addr: PhysAddr) -> Self {
        addr.0 as usize
    }
}

impl From<Pfn> for usize {
    fn from(addr: Pfn) -> Self {
        addr.0 as usize
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
        addr.0 as usize
    }
}

impl From<u64> for PhysAddr {
    fn from(addr: u64) -> Self {
        Self(addr)
    }
}

impl From<PhysAddr> for u64 {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl From<Pfn> for u64 {
    fn from(addr: Pfn) -> Self {
        addr.0
    }
}

impl From<VirtAddr> for u64 {
    fn from(addr: VirtAddr) -> Self {
        addr.0
    }
}

impl From<u64> for Pfn {
    fn from(addr: u64) -> Self {
        Pfn(addr)
    }
}

impl From<usize> for Pfn {
    fn from(addr: usize) -> Self {
        Pfn(addr as u64)
    }
}

impl From<u64> for VirtAddr {
    fn from(addr: u64) -> Self {
        Self(addr)
    }
}

impl From<usize> for VirtAddr {
    fn from(addr: usize) -> Self {
        Self(addr as u64)
    }
}

impl Sub for Pfn {
    type Output = u64;

    fn sub(self, other: Self) -> Self::Output {
        self.0 - other.0
    }
}

impl PhysAddr {
    pub const fn get(&self) -> u64 {
        self.0
    }

    pub const fn to_pfn(&self) -> u64 {
        self.0 >> arch::PAGE_SHIFT
    }

    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }
}

impl Pfn {
    pub const fn get(&self) -> u64 {
        self.0
    }

    pub const fn new(pfn: u64) -> Self {
        Self(pfn)
    }
}

impl VirtAddr {
    pub const fn get(&self) -> u64 {
        self.0
    }

    pub fn from_raw<T>(ptr: *const T) -> Self {
        Self(ptr as u64)
    }

    pub fn to_raw<T>(&self) -> *const T {
        self.0 as *const T
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
