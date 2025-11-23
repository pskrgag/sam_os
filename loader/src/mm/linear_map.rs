use super::page_table::{PageKind, PagePerms, PageTable};
use crate::mm::regions::whole_ram;
use loader_protocol::{LoaderArg, VmmLayoutKind};
use hal::arch::PAGE_SIZE;
use hal::address::{Address, MemRange, VirtAddr};

pub fn map_linear(table: &mut PageTable, prot: &LoaderArg) {
    let res = whole_ram();
    let (base, size) = prot.get_vmm_base(VmmLayoutKind::LinearMap).unwrap();

    assert!(size >= res.count * PAGE_SIZE);

    table.map_pages(
        MemRange::new(
            VirtAddr::new(base.bits() + res.start.bits()),
            res.count * PAGE_SIZE,
        ),
        MemRange::new(res.start, res.count * PAGE_SIZE),
        PagePerms::ReadWrite,
        PageKind::Normal,
    );
}
