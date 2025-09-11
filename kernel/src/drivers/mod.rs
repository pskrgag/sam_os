use loader_protocol::{DeviceKind, LoaderArg};

#[macro_use]
pub mod mmio_mapper;
pub mod irq;
pub mod timer;
pub mod uart;

pub fn init(arg: &LoaderArg) {
    uart::remap(arg.get_device(DeviceKind::Uart).unwrap().0);

    println!("HELLO");
    mmio_mapper::init();

    #[cfg(target_arch = "aarch64")]
    irq::gic::init();

    timer::init();
}
