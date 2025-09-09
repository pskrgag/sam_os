use super::{UartProbe, UARTS};
use core::fmt::{Arguments, Result, Write};
use fdt::node::FdtNode;
use linkme::distributed_slice;
use rtl::uart::{arm_uart::Uart, UartTrait};
use rtl::vmm::types::VirtAddr;

use core::cell::UnsafeCell;

pub struct FakeLock<T> {
    val: UnsafeCell<T>,
}

impl<T> FakeLock<T> {
    pub const fn new(val: T) -> Self {
        Self {
            val: UnsafeCell::new(val),
        }
    }

    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.val.get() }
    }
}

unsafe impl<T> Sync for FakeLock<T> {}

struct Pl031(Uart);

static BACKEND: FakeLock<Pl031> = FakeLock::new(Pl031(Uart::default(VirtAddr::new(0))));

fn probe(node: &FdtNode) -> Option<*mut dyn Write> {
    let mut reg = node.reg()?;
    let reg = reg.next().unwrap();

    *BACKEND.get() = Pl031(Uart::default(reg.starting_address.into()));
    Some(BACKEND.get())
}

pub fn probe_hack() -> Option<*mut dyn Write> {
    *BACKEND.get() = Pl031(Uart::default(0x09000000.into()));
    Some(BACKEND.get())
}

pub fn pprint(args: Arguments) {
    BACKEND.get().write_fmt(args).unwrap()
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
};
