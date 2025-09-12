/// Virtual layout for the kernel

use crate::kernel::kernel_ep;
use loader_protocol::{LoaderArg, VmmLayoutEntry, VmmLayoutKind};
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::Address;

const IMAGE_SIZE: usize = 20 << 30;
const MMIO_SIZE: usize = 1 << 30;
const FIXMAP_SIZE: usize = 1 << 30;
const LOADER_ARG_SIZE: usize = PAGE_SIZE;
const VM_ALLOC_SIZE: usize = 30 << 30;

struct AddressAllocator(usize);

impl AddressAllocator {
    fn alloc(&mut self, size: usize) -> usize {
        let res = self.0;

        self.0 += size;
        res
    }
}

pub fn init_layout(arg: &mut LoaderArg) {
    let mut allocator = AddressAllocator(kernel_ep().bits());

    arg.vmm_layout
        .push(VmmLayoutEntry {
            base: allocator.alloc(IMAGE_SIZE),
            size: IMAGE_SIZE,
            kind: VmmLayoutKind::Image,
        })
        .unwrap();

    arg.vmm_layout
        .push(VmmLayoutEntry {
            base: allocator.alloc(MMIO_SIZE),
            size: MMIO_SIZE,
            kind: VmmLayoutKind::Mmio,
        })
        .unwrap();

    arg.vmm_layout
        .push(VmmLayoutEntry {
            base: allocator.alloc(FIXMAP_SIZE),
            size: FIXMAP_SIZE,
            kind: VmmLayoutKind::Fixmap,
        })
        .unwrap();

    arg.vmm_layout
        .push(VmmLayoutEntry {
            base: allocator.alloc(LOADER_ARG_SIZE),
            size: LOADER_ARG_SIZE,
            kind: VmmLayoutKind::LoaderArg,
        })
        .unwrap();

    arg.vmm_layout
        .push(VmmLayoutEntry {
            base: allocator.alloc(VM_ALLOC_SIZE),
            size: LOADER_ARG_SIZE,
            kind: VmmLayoutKind::VmAlloc,
        })
        .unwrap();
}
