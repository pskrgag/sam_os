/// Virtual layout for the kernel
use loader_protocol::{LoaderArg, VmmLayoutEntry, VmmLayoutKind};

static KERNEL_LAYOUT: [VmmLayoutEntry; VmmLayoutKind::Count as usize] = [
    VmmLayoutEntry {
        base: 0xFFFF700000000000,
        size: 0x100000000000,
        kind: VmmLayoutKind::LinearMap,
    },
    VmmLayoutEntry {
        base: 0xFFFF800000000000,
        size: 0x100000000000,
        kind: VmmLayoutKind::Image,
    },
    VmmLayoutEntry {
        base: 0xFFFF900000000000,
        size: 0x100000000000,
        kind: VmmLayoutKind::Mmio,
    },
    VmmLayoutEntry {
        base: 0xFFFFB00000000000,
        size: 0x100000000000,
        kind: VmmLayoutKind::LoaderArg,
    },
    VmmLayoutEntry {
        base: 0xFFFFC00000000000,
        size: 0x100000000000,
        kind: VmmLayoutKind::VmAlloc,
    },
];

pub fn init_layout(arg: &mut LoaderArg) {
    arg.vmm_layout.extend(KERNEL_LAYOUT.clone());
}
