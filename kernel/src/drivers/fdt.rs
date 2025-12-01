use hal::address::{LinearAddr, PhysAddr};
use loader_protocol::LoaderArg;
use spin::Once;

pub struct Fdt {
    pub base: LinearAddr,
    pub size: usize,
}

pub static FDT: Once<Fdt> = Once::new();

pub fn init(arg: &LoaderArg) {
    FDT.call_once(|| Fdt {
        base: LinearAddr::from(PhysAddr::new(arg.fdt_base)),
        size: arg.fdt_size,
    });
}

pub fn fdt() -> &'static Fdt {
    unsafe { FDT.get_unchecked() }
}
