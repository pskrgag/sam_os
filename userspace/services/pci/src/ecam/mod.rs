use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ptr;
use fdt::Fdt;
use hal::address::{MemRange, PhysAddr, VirtAddr, VirtualAddress};
use libc::vmm::vms::vms;
use pci_types::{Bar, CommandRegister, ConfigRegionAccess, EndpointHeader, PciAddress, PciHeader};
use rtl::error::ErrorType;

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
struct PciMemRange {
    kind: AddressSpace,
    cpu_base: usize,
    size: usize,
    offset: usize,
}

pub struct PciEcam {
    base: VirtAddr,
    ranges: Vec<PciMemRange>,
    devices: BTreeMap<(u16, u16), (u8, u8)>,
}

impl PciMemRange {
    fn allocate(&mut self, size: usize) -> Option<usize> {
        if size > self.size - self.offset {
            None
        } else {
            let res = self.cpu_base + self.offset;

            self.offset += size;
            Some(res)
        }
    }
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

        let ranges: Vec<_> = node
            .ranges()
            .ok_or(ErrorType::InvalidArgument)?
            .map(|r| {
                let kind = ((r.child_bus_address_hi >> 24) & 0b11) as u8;

                PciMemRange {
                    offset: 0,
                    kind: kind.try_into().unwrap(),
                    size: r.size,
                    cpu_base: r.parent_bus_address,
                }
            })
            .collect();

        let mut new = Self {
            base: vms().map_phys(MemRange::new(
                (reg.starting_address as usize).into(),
                reg.size.ok_or(ErrorType::InvalidArgument)?,
            ))?,
            ranges,
            devices: BTreeMap::new(),
        };

        new.enumerate();
        Ok(new)
    }

    fn for_each_bar<F: FnMut(&mut EndpointHeader, Bar, u8, &mut Self)>(
        &mut self,
        bus: u8,
        dev: u8,
        mut f: F,
    ) {
        let address = PciAddress::new(0, bus, dev, 0);
        let header = PciHeader::new(address);

        if let Some(mut endpoint) = EndpointHeader::from_header(header, &*self) {
            let mut slot = 0;

            while slot < 6 {
                if let Some(bar) = endpoint.bar(slot, &*self) {
                    let old_slot = slot;

                    match bar {
                        Bar::Memory64 { .. } => slot += 2,
                        _ => slot += 1,
                    };

                    f(&mut endpoint, bar, old_slot, self);
                } else {
                    slot += 1;
                }
            }
        }
    }

    fn enumerate(&mut self) {
        for bus in 0..=255 {
            for dev in 0..32 {
                let address = PciAddress::new(0, bus, dev, 0);
                let header = PciHeader::new(address);
                let (vendor, device) = header.id(&*self);

                if vendor == u16::MAX {
                    continue;
                }

                self.devices.insert((vendor, device), (bus, dev));
                self.for_each_bar(bus, dev, |endpoint, bar, slot, access| {
                    let addr = access.ranges.iter_mut().find_map(|x| match bar {
                        Bar::Memory64 { size, .. } => {
                            if x.kind != AddressSpace::MemorySpace64 {
                                return None;
                            }

                            x.allocate(size as usize)
                        }
                        Bar::Memory32 { size, .. } => {
                            if x.kind != AddressSpace::MemorySpace32 {
                                return None;
                            }

                            x.allocate(size as usize)
                        }
                        _ => None,
                    });

                    if let Some(addr) = addr {
                        unsafe {
                            endpoint.update_command(&*access, |_| {
                                CommandRegister::MEMORY_ENABLE | CommandRegister::BUS_MASTER_ENABLE
                            });

                            endpoint.write_bar(slot, &*access, addr).unwrap()
                        };
                    }
                });
            }
        }
    }

    pub fn mapping_address(&mut self, vendor: u16, device: u16) -> Option<Vec<MemRange<PhysAddr>>> {
        let (bus, dev) = self.devices.get(&(vendor, device))?;
        let mut mappings = Vec::new();

        self.for_each_bar(*bus, *dev, |_, bar, _, _| {
            let (base, size) = bar.unwrap_mem();

            mappings.push(MemRange::new(base.into(), size));
        });

        Some(mappings)
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
