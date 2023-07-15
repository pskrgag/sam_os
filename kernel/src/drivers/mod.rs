#[macro_use]
pub mod mmio_mapper;
pub mod timer;
pub mod uart;
pub mod irq;

pub fn init() {
    mmio_mapper::init();
    irq::gic::init();
    timer::init();
    uart::remap();
}
