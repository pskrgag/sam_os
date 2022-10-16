use core::fmt;
use crate::arch::*;

#[derive(Clone, Copy)]
pub struct PhysAddr(usize);

#[derive(Clone, Copy)]
pub struct VirtAddr(usize);

pub struct MemRange<T> {
    start: T,
    size: usize
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

impl From<VirtAddr> for usize {
    fn from(addr: VirtAddr) -> Self {
        addr.0
    }
}

impl From<usize> for VirtAddr {
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl PhysAddr {
    pub const fn get(&self) -> usize {
        self.0
    }

    pub const fn to_pfn(&self) -> usize {
        self.0 >> 12/* acrh::PAGE_SHIFT */
    }
}

impl VirtAddr {
    pub const fn get(&self) -> usize {
        self.0
    }

    pub fn from_raw<T>(ptr: *const T) -> Self {
        Self(ptr as usize)
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
