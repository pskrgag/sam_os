use crate::sync::Spinlock;
use arm_gic::{
    gicv3::{GicCpuInterface, GicV3, Group, InterruptGroup},
    IntId, UniqueMmioPointer,
};
use core::ptr::NonNull;
use loader_protocol::{DeviceKind, LoaderArg};
use spin::Once;
use hal::address::VirtualAddress;

pub struct Gic(GicV3<'static>);

pub static GIC: Once<Spinlock<Gic>> = Once::new();
#[derive(Debug)]
pub struct ClaimedIrq(pub IntId);

impl Gic {
    pub fn new(arg: &LoaderArg) -> Self {
        let redist = arg.get_device(DeviceKind::GicRedist).unwrap();
        let dist = arg.get_device(DeviceKind::GicDist).unwrap();

        let gicd = unsafe { UniqueMmioPointer::new(NonNull::new(dist.0.to_raw_mut()).unwrap()) };
        let gicr = NonNull::new(redist.0.to_raw_mut()).unwrap();
        let mut gic = unsafe { GicV3::new(gicd, gicr, 1, false) };

        // Enable interrupts of all prios
        GicCpuInterface::set_priority_mask(0xff);

        // Initialise the GIC on BS CPU
        gic.setup(0);
        Self(gic)
    }

    pub fn enable_irq(&mut self, num: IntId) {
        self.0.set_interrupt_priority(num, Some(0), 0x80).unwrap();
        self.0.set_group(num, Some(0), Group::Group1NS).unwrap();
        self.0
            .set_trigger(num, Some(0), arm_gic::Trigger::Level)
            .unwrap();
        self.0.enable_interrupt(num, Some(0), true).unwrap();
    }

    pub fn pending(&self) -> Option<ClaimedIrq> {
        GicCpuInterface::get_pending_interrupt(InterruptGroup::Group1).map(ClaimedIrq)
    }
}

impl Drop for ClaimedIrq {
    fn drop(&mut self) {
        GicCpuInterface::end_interrupt(self.0, InterruptGroup::Group1)
    }
}

pub fn init(arg: &LoaderArg) {
    GIC.call_once(|| {
        let res = Spinlock::new(Gic::new(arg));

        arm_gic::irq_enable();
        res
    });

    info!("Gic initalized\n");
}
