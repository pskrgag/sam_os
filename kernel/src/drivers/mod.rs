use loader_protocol::{DeviceKind, LoaderArg};

pub mod irq;
pub mod timer;
pub mod uart;

pub fn init_logging(arg: &LoaderArg) {
    uart::remap(arg.get_device(DeviceKind::Uart).unwrap().0);
}

pub fn init(_arg: &LoaderArg) {
    timer::init();
}
