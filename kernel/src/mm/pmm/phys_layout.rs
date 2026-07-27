//! Physical layout
use hal::address::{MemRange, Pfn, PhysAddr};
use heapless::Vec;
use loader_protocol::LoaderArg;
use loader_protocol::MAX_PMM_REGIONS;

static mut PHYS_INFO: PhysInfo = PhysInfo::default();

pub struct PhysInfo {
    layout: Vec<MemRange<PhysAddr>, MAX_PMM_REGIONS>,
    lowest: Pfn,
    highest: Pfn,
}

impl PhysInfo {
    pub const fn default() -> Self {
        Self {
            layout: Vec::new(),
            lowest: unsafe { Pfn::new(0) },
            highest: unsafe { Pfn::new(0) },
        }
    }

    pub fn lowest_pfn(&self) -> Pfn {
        self.lowest
    }

    pub fn highest_pfn(&self) -> Pfn {
        self.highest
    }

    pub fn regions(&self) -> &[MemRange<PhysAddr>] {
        &self.layout
    }
}

pub fn phys_info() -> &'static PhysInfo {
    unsafe { &*(&raw const PHYS_INFO) }
}

pub fn init(arg: &LoaderArg) {
    unsafe {
        let lowest = arg
            .pmm_layout
            .iter()
            .map(|x| x.start().pfn())
            .min()
            .unwrap();

        let highest = arg
            .pmm_layout
            .iter()
            .map(|x| (x.start() + x.size()).pfn())
            .max()
            .unwrap();

        PHYS_INFO = PhysInfo {
            layout: arg.pmm_layout.clone(),
            lowest,
            highest,
        };
    }
}
