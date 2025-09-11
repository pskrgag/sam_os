use loader_protocol::LoaderArg;
use fdt::Fdt;

pub mod uart;

pub fn map(fdt: &Fdt, arg: &mut LoaderArg) {
    uart::map(fdt, arg);
}
