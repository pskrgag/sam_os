#[macro_use]
pub mod gic;
pub mod irq;
pub mod mmio_mapper;
pub mod timer;
pub mod uart;

pub fn init() {
    mmio_mapper::init();
    gic::init();
    timer::init();
    uart::remap();
}
