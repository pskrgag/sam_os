use fdt::Fdt;
use hal::address::{MemRange, PhysAddr};
use hal::uart::{pl011::Uart, UartTrait};
use libc::vmm::vms::vms;

pub struct Pl011(Uart);

impl Pl011 {
    pub fn read_byte(&mut self) -> u8 {
        let mut bytes = [0u8; 1];

        self.0.read_bytes(&mut bytes);
        bytes[0]
    }

    pub fn write_byte(&mut self, byte: u8) {
        let bytes = [byte; 1];

        self.0.write_bytes(&bytes);
    }
}

pub fn probe(fdt: &Fdt) -> Option<Pl011> {
    let aliases = fdt.aliases()?;
    let mut realname = None;

    // Hacky, but use serial1, since serial0 is occupied by the kernel
    // TODO: kernel should protect mapped regions
    for alias in aliases.all() {
        if alias.0 == "serial1" {
            realname = Some(alias.1);
        }
    }

    let node = fdt.find_node(realname?)?;
    let _ = node.compatible()?.all().find(|x| *x == "arm,pl011")?;

    // It's pl011 node. Get the base address
    let reg = node.reg()?.next()?;

    // Map it
    let res = vms().map_phys(MemRange::new(
        PhysAddr::new(reg.starting_address as usize),
        reg.size?,
    )).ok()?;

    Some(Pl011(Uart::new(res)))
}
