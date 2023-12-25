use crate::arch::{self, PAGE_SIZE};
use core::ops::Add;
use core::{
    fmt::{self, Debug},
    ops::Sub,
};
use shared::vmm::types::*;

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
