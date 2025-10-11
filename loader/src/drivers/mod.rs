use fdt::Fdt;
use loader_protocol::LoaderArg;

pub mod irq;
pub mod uart;

pub fn map(fdt: &Fdt, arg: &mut LoaderArg) {
    uart::map(fdt, arg);
    irq::map(fdt, arg);
}
