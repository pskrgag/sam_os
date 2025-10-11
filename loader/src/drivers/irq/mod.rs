use fdt::{Fdt, node::FdtNode};
use linkme::distributed_slice;
use loader_protocol::LoaderArg;

pub mod gic;

struct IrqProbe {
    pub compatible: &'static str,
    pub map: fn(&FdtNode, &mut LoaderArg),
}

#[distributed_slice]
pub static IRQS: [IrqProbe];

pub fn map(fdt: &Fdt, arg: &mut LoaderArg) {
    for irq in IRQS {
        if let Some(node) = fdt.find_compatible(&[irq.compatible]) {
            (irq.map)(&node, arg);
        }
    }
}
