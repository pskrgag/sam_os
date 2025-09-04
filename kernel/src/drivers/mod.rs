#[macro_use]
pub mod mmio_mapper;
pub mod irq;
pub mod timer;
pub mod uart;

pub fn init() {
    mmio_mapper::init();

    #[cfg(target_arch = "aarch64")]
    irq::gic::init();

    timer::init();
    uart::remap();
}
