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
    pub fn get(&self) -> usize {
        self.0
    }

    pub const fn to_pfn(&self) -> usize {
        self.0 >> 12/* acrh::PAGE_SHIFT */
    }
}

impl VirtAddr {
    pub fn get(&self) -> usize {
        self.0
    }

    /* D5.2 The VMSAv8-64 address translation system */
    pub fn is_valid_kernel_addr(&self) -> bool {
        if self.0 < 0xFFFF_FFFF_FFFF_FFFF && self.0 > 0xFFFF_0000_0000_0000 {
            true
        } else {
            false
        }
    }
}
