use core::ops::DerefMut;
use fdt::Fdt;
use heapless::Vec;
use hal::arch::{PAGE_SHIFT, PAGE_SIZE};
use rtl::linker_var;
use rtl::locking::fakelock::FakeLock;
use hal::address::{Address, PhysAddr};

unsafe extern "C" {
    static __start: usize;
    static __end: usize;
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub count: usize,
}

impl MemoryRegion {
    fn contains(&self, addr: PhysAddr) -> bool {
        let start = self.start.bits();
        let addr = addr.bits();
        let size = self.count * PAGE_SIZE;

        addr >= start && addr < start + size
    }

    fn exclude(&mut self, reg_start: PhysAddr, reg_size: usize) -> Option<Self> {
        assert!(self.contains(reg_start));

        let start = self.start.bits();
        let reg_start = reg_start.bits();
        let size = self.count * PAGE_SIZE;
        let end = start + size;

        assert!(size.is_page_aligned());
        assert!(reg_start.is_page_aligned());

        if reg_start == start {
            self.start = PhysAddr::new(start + size);
            self.count -= reg_size / PAGE_SIZE;
            None
        } else if reg_start + reg_size == end {
            self.count -= reg_size / PAGE_SIZE;
            None
        } else {
            self.count = (reg_start - start) / PAGE_SIZE;

            Some(MemoryRegion {
                start: PhysAddr::new(reg_start + reg_size),
                count: (end - (reg_start + reg_size)) / PAGE_SIZE,
            })
        }
    }
}

static ALLOC_MEM_REGIONS: FakeLock<Vec<MemoryRegion, 3>> = FakeLock::new(Vec::new());
static PHYSICAL_REGION: FakeLock<Option<MemoryRegion>> = FakeLock::new(None);

fn add_region(reg: MemoryRegion) {
    ALLOC_MEM_REGIONS.get().push(reg).unwrap();
}

pub fn init(fdt: &Fdt) {
    let mem = fdt.memory();
    let image_start = PhysAddr::new(linker_var!(__start));
    let image_size = linker_var!(__end) - image_start.bits();

    for reg in mem.regions() {
        println!(
            "Found free memory region [start: {:x}, size: {:x}]",
            reg.starting_address as usize,
            reg.size.unwrap()
        );

        let mut reg = MemoryRegion {
            start: (reg.starting_address as usize).into(),
            count: reg.size.unwrap() >> PAGE_SHIFT,
        };

        assert!(PHYSICAL_REGION.get().is_none());
        *PHYSICAL_REGION.get() = Some(reg.clone());

        if reg.contains(image_start) {
            reg.exclude(image_start, image_size).map(|x| add_region(x));
        }

        add_region(reg);
    }
}

pub fn whole_ram() -> MemoryRegion {
    PHYSICAL_REGION.get().clone().unwrap()
}

pub fn regions() -> &'static mut Vec<MemoryRegion, 3> {
    unsafe { &mut *(ALLOC_MEM_REGIONS.get().deref_mut() as *mut _) }
}
