use heapless::Vec;
use hal::address::*;
use spin::Once;

use loader_protocol::{LoaderArg, MAX_VMM_REGIONS, VmmLayoutEntry, VmmLayoutKind};

static VMM_LAYOUT: Once<Vec<VmmLayoutEntry, MAX_VMM_REGIONS>> = Once::new();

pub fn vmm_range(kind: VmmLayoutKind) -> MemRange<VirtAddr> {
    let entry = &VMM_LAYOUT.get().unwrap()[kind as usize];

    MemRange::new(VirtAddr::from(entry.base), entry.size)
}

pub fn init(arg: &LoaderArg) {
    VMM_LAYOUT.call_once(|| arg.vmm_layout.clone());
}
