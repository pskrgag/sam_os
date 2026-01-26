use super::{IRQS, IrqProbe};
use fdt::node::FdtNode;
use linkme::distributed_slice;
use loader_protocol::{DeviceKind, DeviceMapping, LoaderArg};

fn map(node: &FdtNode, arg: &mut LoaderArg) {
    let mut iter = node.reg().unwrap();
    let reg = iter.next().unwrap();

    arg.devices
        .push(DeviceMapping {
            base: reg.starting_address as usize,
            size: reg.size.unwrap(),
            kind: DeviceKind::GicDist,
        })
        .expect("Too many devices");

    let reg = iter.next().expect("Expect REDIST and DIST in GIC FDT Node");

    arg.devices
        .push(DeviceMapping {
            base: reg.starting_address as usize,
            size: reg.size.unwrap(),
            kind: DeviceKind::GicRedist,
        })
        .expect("Too many devices");

    info!("Mapped gic-v3\n");
}

#[distributed_slice(IRQS)]
static GIC: IrqProbe = IrqProbe {
    compatible: "arm,gic-v3",
    map,
};
