use rtl::vmm::types::*;

pub const fn uart_base() -> VirtAddr {
    VirtAddr::new(0x09000000)
}

pub const fn ram_base() -> PhysAddr {
    PhysAddr::new(0x0400000)
}

pub const fn ram_size() -> usize {
    0x02000000
}
