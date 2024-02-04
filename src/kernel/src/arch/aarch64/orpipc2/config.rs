use crate::arch;
use rtl::vmm::types::*;

pub const fn uart_base() -> VirtAddr {
    VirtAddr::new(0x01C2_8000)
}

pub const fn ram_base() -> PhysAddr {
    PhysAddr::new(0x0400000)
}

pub const fn ram_size() -> usize {
    0x02000000
}

pub const fn gic_dist() -> (PhysAddr, usize) {
    (PhysAddr::new(0x01c81000), 0x1000)
}

pub const fn gic_cpu() -> (PhysAddr, usize) {
    (PhysAddr::new(0x01c82000), 0x1000)
}
