use super::IrqController;
use arm_gic::{
    gicv3::{GicCpuInterface, GicV3, Group, InterruptGroup},
    IntId, UniqueMmioPointer,
};
use core::ptr::NonNull;
use hal::address::VirtualAddress;
use loader_protocol::{DeviceKind, LoaderArg};
use rtl::irq::IrqTrigger;
use rtl::locking::spinlock::Spinlock;
use spin::Once;

pub struct Gic(GicV3<'static>);

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
}

impl Drop for ClaimedIrq {
    fn drop(&mut self) {
        GicCpuInterface::end_interrupt(self.0, InterruptGroup::Group1)
    }
}

impl IrqController for Spinlock<Gic> {
    fn enable_irq(&self, num: IntId, trigger: IrqTrigger) {
        let trigger = match trigger {
            IrqTrigger::Edge => arm_gic::Trigger::Edge,
            IrqTrigger::Level => arm_gic::Trigger::Level,
        };
        let mut gic = self.lock();

        gic.0.set_interrupt_priority(num, Some(0), 0x80).unwrap();
        gic.0.set_group(num, Some(0), Group::Group1NS).unwrap();
        gic.0.set_trigger(num, Some(0), trigger).unwrap();
        gic.0.enable_interrupt(num, Some(0), true).unwrap();
    }

    fn mask_irq(&self, num: super::IntId, mask: bool) {
        let mut gic = self.lock();

        gic.0.enable_interrupt(num, Some(0), !mask).unwrap();
    }

    fn pending(&self) -> Option<IntId> {
        GicCpuInterface::get_pending_interrupt(InterruptGroup::Group1)
    }

    fn eoi(&self, int: IntId) {
        GicCpuInterface::end_interrupt(int, InterruptGroup::Group1)
    }
}

pub fn init(arg: &LoaderArg) {
    static GIC: Once<Spinlock<Gic>> = Once::new();
    let gic = GIC.call_once(|| Spinlock::new(Gic::new(arg)));

    super::register_controller(gic);
    arm_gic::irq_enable();
}
