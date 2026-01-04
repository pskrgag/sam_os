use super::{UARTS, UartProbe};
use core::fmt::{Result, Write};
use core::mem::MaybeUninit;
use fdt::node::FdtNode;
use hal::arch::PAGE_SIZE;
use hal::uart::{UartTrait, pl011::Uart};
use linkme::distributed_slice;
use loader_protocol::{DeviceKind, DeviceMapping, LoaderArg};
use rtl::locking::fakelock::FakeLock;

struct Pl031(Uart);

static BACKEND: FakeLock<MaybeUninit<Pl031>> = FakeLock::new(MaybeUninit::uninit());

fn probe(node: &FdtNode) -> Option<*mut dyn Write> {
    let mut reg = node.reg()?;
    let reg = reg.next().unwrap();

    *BACKEND.get() = MaybeUninit::new(Pl031(Uart::new(reg.starting_address.into())));
    Some(unsafe { BACKEND.get().assume_init_mut() })
}

fn map(node: &FdtNode, arg: &mut LoaderArg) {
    let mut reg = node.reg().unwrap();
    let reg = reg.next().unwrap();

    arg.devices
        .push(DeviceMapping {
            base: reg.starting_address as usize,
            size: PAGE_SIZE,
            kind: DeviceKind::Uart,
        })
        .expect("Too many devices");

    println!("Mapped pl011");
}

impl Write for Pl031 {
    fn write_str(&mut self, s: &str) -> Result {
        self.0.write_bytes(s.as_bytes());
        Ok(())
    }
}

#[distributed_slice(UARTS)]
static PL011: UartProbe = UartProbe {
    compatible: "arm,pl011",
    probe,
    map,
};
