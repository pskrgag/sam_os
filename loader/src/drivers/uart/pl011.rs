use super::{UartProbe, UARTS};
use core::fmt::{Result, Write};
use fdt::node::FdtNode;
use linkme::distributed_slice;
use loader_protocol::{DeviceKind, DeviceMapping, LoaderArg};
use rtl::arch::PAGE_SIZE;
use rtl::locking::fakelock::FakeLock;
use rtl::uart::{arm_uart::Uart, UartTrait};
use rtl::vmm::types::VirtAddr;

struct Pl031(Uart);

static BACKEND: FakeLock<Pl031> = FakeLock::new(Pl031(Uart::default(VirtAddr::new(0))));

fn probe(node: &FdtNode) -> Option<*mut dyn Write> {
    let mut reg = node.reg()?;
    let reg = reg.next().unwrap();

    *BACKEND.get() = Pl031(Uart::default(reg.starting_address.into()));
    Some(BACKEND.get())
}

fn map(node: &FdtNode, arg: &mut LoaderArg) {
    let mut reg = node.reg().unwrap();
    let reg = reg.next().unwrap();

    arg.devices.push(DeviceMapping {
        base: reg.starting_address as usize,
        size: PAGE_SIZE,
        kind: DeviceKind::Uart,
    }).expect("Too many devices");

    println!("Mapped pl031");
}

impl Write for Pl031 {
    fn write_str(&mut self, s: &str) -> Result {
        self.0.write_bytes(s.as_bytes());
        Ok(())
    }
}

#[distributed_slice(UARTS)]
static PL031: UartProbe = UartProbe {
    compatible: "arm,pl011",
    probe,
    map,
};
