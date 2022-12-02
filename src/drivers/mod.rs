#[macro_use]
pub mod gic;
pub mod mmio_mapper;
pub mod uart;
pub mod timer;
pub mod irq;

pub fn init() {
    mmio_mapper::init();
    gic::init();
    timer::init(1000);
}
