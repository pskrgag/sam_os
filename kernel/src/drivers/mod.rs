use loader_protocol::{DeviceKind, LoaderArg};

pub mod irq;
pub mod timer;
pub mod uart;

pub fn init_logging(arg: &LoaderArg) {
    uart::remap(arg.get_device(DeviceKind::Uart).unwrap().0);
}

pub fn init(arg: &LoaderArg) {

    println!("HELLO");

    #[cfg(target_arch = "aarch64")]
    irq::gic::init(arg);

    timer::init();
}
