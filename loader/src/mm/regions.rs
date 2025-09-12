use crate::linker_var;
use fdt::Fdt;
use heapless::Vec;
use rtl::arch::{PAGE_SHIFT, PAGE_SIZE};
use rtl::locking::fakelock::FakeLock;
use rtl::vmm::types::{Address, PhysAddr};
use core::ops::DerefMut;

unsafe extern "C" {
    static __start: usize;
    static __end: usize;
}

#[derive(Debug)]
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

static MEM_REGIONS: FakeLock<Vec<MemoryRegion, 10>> = FakeLock::new(Vec::new());

fn add_region(reg: MemoryRegion) {
    println!(
        "Found free memory region [start: {:x}, pages: {:x}]",
        reg.start, reg.count
    );
    MEM_REGIONS.get().push(reg).unwrap();
}

pub fn init(fdt: &Fdt) {
    let mem = fdt.memory();
    let image_start = PhysAddr::new(linker_var!(__start));
    let image_size = linker_var!(__end) - image_start.bits();

    for reg in mem.regions() {
        let mut reg = MemoryRegion {
            start: (reg.starting_address as usize).into(),
            count: reg.size.unwrap() >> PAGE_SHIFT,
        };

        if reg.contains(image_start) {
            reg.exclude(image_start, image_size).map(|x| add_region(x));
        }

        add_region(reg);
    }
}

pub fn regions() -> &'static mut Vec<MemoryRegion, 10> {
    unsafe { &mut *(MEM_REGIONS.get().deref_mut() as *mut _) }
}
