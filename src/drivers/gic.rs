// FIXME one day...
#[path = "../arch/aarch64/qemu/config.rs"]
mod config;

use crate::{
    drivers::mmio_mapper::MMIO_ALLOCATOR,
    kernel::{locking::spinlock::Spinlock, misc::num_pages},
    mm::types::*,
};

const GICD_CTLR: usize = 0x0;
const GICC_CTLR: usize = 0x0;

// FIXME one day...
use crate::{print, println};

// GICv2 only!
struct Gic {
    dist_va: VirtAddr,
    cpu_va: VirtAddr,
}

static GIC: Spinlock<Gic> = Spinlock::new(Gic::new());

impl Gic {
    pub const fn new() -> Self {
        Self {
            dist_va: VirtAddr::new(0),
            cpu_va: VirtAddr::new(0),
        }
    }

    fn write_to_reg(base: VirtAddr, off: usize, val: usize) {
        unsafe {
            (base.to_raw_mut::<u8>().offset(off as isize) as *mut usize).write_volatile(val);
        }
    }

    fn read_reg(base: VirtAddr, off: usize) -> usize {
        unsafe {
            (base.to_raw_mut::<u8>().offset(off as isize) as *const usize).read_volatile()
        }
    }

    fn init(&mut self) -> Option<()> {
        let dist = config::gic_dist();
        let cpu = config::gic_cpu();

        self.cpu_va = MMIO_ALLOCATOR.get().iomap(cpu.0, num_pages(cpu.1))?;
        self.dist_va = MMIO_ALLOCATOR.get().iomap(dist.0, num_pages(dist.1))?;

        unsafe { core::arch::asm!("tlbi vmalle1") };
        unsafe { core::arch::asm!("dsb ish") };
        unsafe { core::arch::asm!("isb") };

        Self::write_to_reg(self.dist_va, GICD_CTLR, 0x1);
        Self::write_to_reg(self.cpu_va, GICC_CTLR, 0x1);

        Some(())
    }
}

pub fn init() {
    GIC.lock().init().expect("Failed to initalize GIC");

    println!("Gic initalized");
}
