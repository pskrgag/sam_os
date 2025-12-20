use core::ptr;
use fdt::Fdt;
use hal::address::{MemRange, VirtAddr, VirtualAddress};
use libc::vmm::vms::vms;
use pci_types::{Bar, ConfigRegionAccess, EndpointHeader, PciAddress, PciHeader};
use rtl::error::ErrorType;

#[derive(Debug)]
enum AddressSpace {
    ConfigSpace,
    IOSpace,
    MemorySpace32,
    MemorySpace64,
}

impl TryFrom<u8> for AddressSpace {
    type Error = ErrorType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ConfigSpace),
            1 => Ok(Self::IOSpace),
            2 => Ok(Self::MemorySpace32),
            3 => Ok(Self::MemorySpace64),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
}

pub struct PciEcam {
    base: VirtAddr,
}

// ranges = <
// 0x1000000  0x00            0x00       0x00       0x3eff0000 0x00    0x10000
// 0x2000000  0x00            0x10000000 0x00       0x10000000 0x00    0x2eff0000
// 0x3000000  0x80            0x00       0x80       0x00       0x80    0x00
// >;
// #size-cells = <0x02>;
// #address-cells = <0x03>;

// phys.hi cell: npt000ss bbbbbbbb dddddfff rrrrrrrr
// phys.mid cell: hhhhhhhh hhhhhhhh hhhhhhhh hhhhhhhh
// phys.low cell: llllllll llllllll llllllll llllllll

impl PciEcam {
    pub fn new(fdt: &Fdt) -> Result<Self, ErrorType> {
        let node = fdt
            .find_compatible(&["pci-host-ecam-generic"])
            .ok_or(ErrorType::NotFound)?;

        let reg = node
            .reg()
            .ok_or(ErrorType::InvalidArgument)?
            .next()
            .ok_or(ErrorType::InvalidArgument)?;
        let ranges = node.ranges().ok_or(ErrorType::InvalidArgument)?;

        for r in ranges {
            let kind = (r.child_bus_address_hi >> 24) & 0b11;
            let space: AddressSpace = (kind as u8).try_into()?;

            println!("{:?}", space);
        }

        Ok(Self {
            base: vms().map_phys(MemRange::new(
                (reg.starting_address as usize).into(),
                reg.size.ok_or(ErrorType::InvalidArgument)?,
            ))?,
        })
    }

    pub fn enumerate(&self) {
        for bus in 0..=255 {
            for dev in 0..32 {
                let address = PciAddress::new(0, bus, dev, 0);
                let header = PciHeader::new(address);
                let (vendor, device) = header.id(self);
                let (_, class, subclass, interface) = header.revision_and_class(self);

                if let Some(endpoint) = EndpointHeader::from_header(header, self) {
                    if vendor == u16::MAX {
                        continue;
                    }

                    println!(
                        "vendor: {:x}, device: {:x}, class: {}, subclass: {}, iface: {}",
                        vendor, device, class, subclass, interface
                    );

                    let mut slot = 0;

                    while slot < 6 {
                        if let Some(bar) = endpoint.bar(slot, self) {
                            if matches!(bar, Bar::Memory64 { .. }) {
                                slot += 2;
                            } else {
                                slot += 1;
                            }
                        } else {
                            slot += 1;
                        }
                    }
                }
            }
        }
    }

    fn translate(&self, pci_address: PciAddress) -> *const u8 {
        unsafe {
            self.base.to_raw::<u8>().add(
                (pci_address.bus() as usize) << 20
                    | (pci_address.device() as usize) << 15
                    | (pci_address.function() as usize) << 12,
            )
        }
    }
}

impl ConfigRegionAccess for PciEcam {
    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        unsafe { ptr::read_volatile(self.translate(address).add(offset as usize) as *const u32) }
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        unsafe {
            ptr::write_volatile(
                self.translate(address).add(offset as usize) as *mut u32,
                value,
            );
        }
    }
}
