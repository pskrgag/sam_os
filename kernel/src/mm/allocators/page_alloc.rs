use crate::{
    arch::{self, PAGE_SHIFT, PAGE_SIZE},
    kernel,
    kernel::locking::fake_lock::FakeLock,
    lib,
    mm::types::*,
};
use spin::once::Once;

extern "C" {
    static end: usize;
}

/// Purpose of this allocator is to allocate memory at the boot time and never de-allocate it
pub struct PageAlloc {
    start: MemRange<PhysAddr>,
    cursor: usize,
}

pub static mut PAGE_ALLOC: Once<PageAlloc> = Once::new();

impl PageAlloc {
    pub fn new(range: MemRange<PhysAddr>) -> Self {
        Self {
            start: range,
            cursor: 0,
        }
    }

    pub fn alloc(&mut self, pages: usize) -> Option<PhysAddr> {
        if self.cursor + pages >= self.start.size() >> PAGE_SHIFT {
            None
        } else {
            let res = self.start.start().get() + self.cursor;

            self.cursor += pages;
            println!("Addr 0x{:x}", res);
            Some(PhysAddr::from(res))
        }
    }
}

pub fn init() {
    let alloc_start = PhysAddr::from(VirtAddr::from(linker_var!(end)));
    let alloc_size = arch::ram_size() as usize - kernel::misc::image_size();

    println!(
        "Page allocator start {:x} size {:x}",
        alloc_start.get(),
        alloc_size
    );

    unsafe {
        PAGE_ALLOC.call_once(|| PageAlloc::new(MemRange::new(alloc_start, alloc_size)));
    }
}

pub fn page_allocator() -> &'static mut PageAlloc {
    unsafe { PAGE_ALLOC.get_mut().unwrap() }
}
