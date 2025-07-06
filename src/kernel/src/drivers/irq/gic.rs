use crate::{arch, drivers::mmio_mapper::MMIO_ALLOCATOR};
use rtl::vmm::types::*;
use crate::percpu_global;

const GICD_CTLR: usize = 0x0;
const GICC_CTLR: usize = 0x0;

// Distributor Registers
const CTLR: usize = 0;
const ISENABLER: usize = 0x0100;
const ICENABLER: usize = 0x0180;
const ICPENDR: usize = 0x0280;
const ICACTIVER: usize = 0x0380;
const ITARGETSR: usize = 0x0800;
const IPRIORITYR: usize = 0x0400;
const ICFGR: usize = 0x0c00;

// Registers VALUES
const CTLR_ENABLE: u32 = 1;
const CTLR_DISABLE: u32 = 0;

const ICENABLER_SHIFT: u32 = 5;
const ICENABLER_MASK: u32 = 0b11111;

const ISENABLER_SHIFT: u32 = 5;
const ISENABLER_MASK: u32 = 0b11111;

const ICPENDR_SHIFT: u32 = 5;
const ICPENDR_MASK: u32 = 0b11111;

// 8 bits per 4 interrupts in register
const ITARGETSR_INTERRUPT_MASK: u32 = 0b11; // interrupt reminder mask
const ITARGETSR_INTERRUPT_SHIFT: u32 = 2; // interrupt divider shift (division by 4)
const ITARGETSR_VALUE_SHIFT: u32 = 3; // core number shift (multiplication by 8)
const ITARGETSR_VALUE_MASK: u32 = 0b111111111; // core number 1 byte mask

const IPRIORITYR_INTERRUPT_MASK: u32 = 0b11; // interrupt reminder mask
const IPRIORITYR_INTERRUPT_SHIFT: u32 = 2; // interrupt divider shift (division by 4)
const IPRIORITYR_VALUE_SHIFT: u32 = 3; // priority number shift (multiplication by 8)
const IPRIORITYR_VALUE_MASK: u32 = 0b111111111; // priority number 1 byte mask

const ICFGR_INTERRUPT_MASK: u32 = 0b1111; // interrupt reminder mask
const ICFGR_INTERRUPT_SHIFT: u32 = 4; // interrupt divider shift (division by 4)
const ICFGR_VALUE_SHIFT: u32 = 1; // config value shift (multiplication by 8)
const ICFGR_VALUE_MASK: u32 = 0b11; // config value 2 bits mask

const IPRIORITY_SIZE: u32 = 4;
const IPRIORITY_BITS: u32 = 8;
const ICFGR_SIZE: u32 = 16;
const ICFGR_BITS: u32 = 2;

// Cpu Registers
const PMR: usize = 0x4;

const PMR_PRIO: u32 = 0xF0;

const IRQ_LINES: u32 = 256;

struct Gicc {
    base: VirtAddr,
}

struct Gicd {
    base: VirtAddr,
}

// GICv2 only!
pub struct Gic {
    cpu: Gicc,
    dist: Gicd,
}

// TODO: Should it be per-cpu locked?
percpu_global!(
    pub static GIC: Gic = Gic::new();
);

#[inline(never)]
fn write_to_reg<T>(base: VirtAddr, offset: usize, val: T) {
    unsafe {
        base.to_raw_mut::<T>()
            .add(offset)
            .write_volatile(val);
    };
}

#[inline(never)]
fn read_from_reg<T>(base: VirtAddr, offset: usize) -> T {
    unsafe {
        base.to_raw_mut::<T>()
            .add(offset)
            .read_volatile()
    }
}

impl Gicc {
    pub const fn default() -> Self {
        Self {
            base: VirtAddr::new(0),
        }
    }

    pub fn new(base: PhysAddr) -> Option<Self> {
        let cpu_va = MMIO_ALLOCATOR.lock().iomap(base, 1)?;
        Some(Self { base: cpu_va })
    }

    pub fn init(&mut self) {
        write_to_reg::<u32>(self.base, 0, 1);
        write_to_reg::<u32>(self.base, PMR >> 2, 0xf0);
        write_to_reg::<u32>(self.base, 8 >> 2, 0x00);
    }
}

impl Gicd {
    pub const fn default() -> Self {
        Self {
            base: VirtAddr::new(0),
        }
    }

    pub fn disable(&mut self) {
        write_to_reg::<u32>(self.base, 0, 0);
    }

    pub fn new(base: PhysAddr) -> Option<Self> {
        let dist_va = MMIO_ALLOCATOR.lock().iomap(base, 1)?;
        Some(Self { base: dist_va })
    }

    pub fn init(&mut self) {
        for i in 0..IRQ_LINES >> 2 {
            write_to_reg(self.base, ICENABLER + i as usize, u32::MAX);
            write_to_reg(self.base, ICPENDR + i as usize, u32::MAX);
            write_to_reg(self.base, ICACTIVER + i as usize, u32::MAX);
        }
    }

    pub fn enable(&mut self) {
        write_to_reg::<u32>(self.base, 0, 1);
    }

    pub fn set_priotity(&mut self, intnum: u32, prio: u32) {
        let shift = (intnum & IPRIORITYR_INTERRUPT_MASK) << IPRIORITYR_VALUE_SHIFT;
        let offset = intnum >> IPRIORITYR_INTERRUPT_SHIFT;
        let value = 0xA0;

        write_to_reg::<u32>(
            self.base,
            (IPRIORITYR >> 2) + offset as usize,
            value | prio << shift,
        );
    }

    pub fn set_interrupt_core(&self, intnum: u32, core: u32) {
        let shift = (intnum & ITARGETSR_INTERRUPT_MASK) << ITARGETSR_VALUE_SHIFT;
        let offset = intnum >> ITARGETSR_INTERRUPT_SHIFT;
        let value = read_from_reg::<u32>(self.base, (ITARGETSR >> 2) + offset as usize)
            & !(ITARGETSR_VALUE_MASK << shift);

        write_to_reg::<u32>(
            self.base,
            (ITARGETSR >> 2) + offset as usize,
            value | (1 << core) << 16,
        );
    }

    pub fn set_interrupt_config(&self, intnum: u32, config: u32) {
        let shift = (intnum & ICFGR_INTERRUPT_MASK) << ICFGR_VALUE_SHIFT;
        let offset = intnum >> ICFGR_INTERRUPT_SHIFT;
        let value = read_from_reg::<u32>(self.base, (ICFGR >> 2) + offset as usize)
            & !(ICFGR_VALUE_MASK << shift);

        write_to_reg::<u32>(
            self.base,
            (ICFGR >> 2) + offset as usize,
            value | config << shift,
        );
    }

    pub fn clear_interrupt(&self, intnum: u32) {
        write_to_reg::<u32>(
            self.base,
            (ICPENDR >> 2) + (intnum as usize >> ICPENDR_SHIFT),
            1 << (intnum & ICPENDR_MASK),
        );
    }

    pub fn set_interrupt(&self, intnum: u32) {
        write_to_reg::<u32>(
            self.base,
            (ISENABLER >> 2) + (intnum as usize >> ISENABLER_SHIFT),
            1 << (intnum & ISENABLER_MASK),
        );
    }

    pub fn is_pending(&self, intnum: u32) -> bool {
        let value = read_from_reg::<u32>(
            self.base,
            (ICPENDR >> 2) + (intnum as usize >> ICPENDR_SHIFT),
        );

        value & (1 << (intnum & ICPENDR_MASK)) != 0
    }
}

impl Gic {
    pub const fn new() -> Self {
        Self {
            dist: Gicd::default(),
            cpu: Gicc::default(),
        }
    }

    fn init(&mut self) -> Option<()> {
        let cpu = arch::gic_cpu();
        let dist = arch::gic_dist();

        self.cpu = Gicc::new(cpu.0)?;
        self.dist = Gicd::new(dist.0)?;

        // Turn off to start initialization
        self.dist.disable();

        self.dist.init();
        self.cpu.init();

        self.dist.enable();

        Some(())
    }

    pub fn enable_irq(&mut self, num: u32) {
        self.dist.set_priotity(num, 0);
        self.dist.set_interrupt_core(num, 0);
        self.dist.clear_interrupt(num);
        self.dist.set_interrupt_config(num, 0);
        self.dist.set_interrupt(num);
    }

    pub fn init_secondary(&mut self, irq: u32, _cpu: u32) {
        self.cpu.init();
        // self.dist.set_priotity(irq, 0);
        // self.dist.set_interrupt_core(irq, cpu);
        // self.dist.set_interrupt_config(irq, 2);
        self.dist.set_interrupt(irq);
    }

    pub fn clear_interrupt(&mut self, irq: u32) {
        self.dist.clear_interrupt(irq);
    }

    pub fn is_pending(&self, intnum: u32) -> bool {
        self.dist.is_pending(intnum)
    }
}

pub fn init() {
    GIC.per_cpu_var_get_mut().init().expect("Failed to initalize GIC");

    println!("Gic initalized");
}
