#[macro_use]
pub mod gic;
pub mod mmio_mapper;
pub mod uart;

pub fn init() {
    mmio_mapper::init();
    gic::init();
}
