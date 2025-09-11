use crate::kernel::mmio_start;
use crate::mm::{
    alloc::alloc_pages,
    page_table::{PageKind, PagePerms, PageTable},
};
use loader_protocol::LoaderArg;
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::{Address, MemRange, PhysAddr, VirtAddr};

pub fn prepare(fdt: PhysAddr, mut arg: LoaderArg, tt: &mut PageTable) -> VirtAddr {
    let mut mmio_start = mmio_start();

    arg.tt_base = tt as *mut _ as usize;
    arg.fdt_base = fdt.bits();

    for dev in &mut arg.devices {
        tt.map_pages(
            MemRange::new(mmio_start, dev.size),
            MemRange::new(PhysAddr::new(dev.base), dev.size),
            PagePerms::ReadWrite,
            PageKind::Device,
        );

        dev.base = mmio_start.bits();
        mmio_start = VirtAddr::new(mmio_start.bits() + dev.size);
    }

    // Map arg page to the kernel
    let page = alloc_pages(1).unwrap();
    *unsafe { &mut *(page.bits() as *mut LoaderArg) } = arg;

    tt.map_pages(
        MemRange::new(mmio_start, PAGE_SIZE),
        MemRange::new(page, PAGE_SIZE),
        PagePerms::Read,
        PageKind::Normal,
    );

    mmio_start
}
