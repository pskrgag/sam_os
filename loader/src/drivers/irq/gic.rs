use super::{IRQS, IrqProbe};
use fdt::node::FdtNode;
use linkme::distributed_slice;
use loader_protocol::{DeviceKind, DeviceMapping, LoaderArg};
use rtl::arch::PAGE_SIZE;

fn map(node: &FdtNode, arg: &mut LoaderArg) {
    let mut iter = node.reg().unwrap();
    let reg = iter.next().unwrap();

    arg.devices
        .push(DeviceMapping {
            base: reg.starting_address as usize,
            size: PAGE_SIZE,
            kind: DeviceKind::GicDist,
        })
        .expect("Too many devices");

    let reg = iter.next().expect("Expect CPU and DIST in GIC FDT Node");

    arg.devices
        .push(DeviceMapping {
            base: reg.starting_address as usize,
            size: PAGE_SIZE,
            kind: DeviceKind::GicCpu,
        })
        .expect("Too many devices");

    println!("Mapped gic");
}

#[distributed_slice(IRQS)]
static PL031: IrqProbe = IrqProbe {
    compatible: "arm,cortex-a15-gic",
    map,
};
