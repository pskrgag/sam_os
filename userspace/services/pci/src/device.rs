use super::ecam::PciEcam;
use crate::bindings_Device::{Device, MapRx, MapTx, PciMapping};
use alloc::boxed::Box;
use alloc::sync::Arc;
use hal::address::Address;
use libc::handle::Handle;
use libc::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub struct PciDevice {
    vendor: u16,
    device: u16,
    bus: Arc<Spinlock<PciEcam>>,
}

impl PciDevice {
    pub fn new(
        vendor: u16,
        device: u16,
        bus: Arc<Spinlock<PciEcam>>,
    ) -> Result<(Box<Device<Self>>, Handle), ErrorType> {
        let port = Port::create()?;
        let val = Self {
            vendor,
            device,
            bus,
        };
        let raw_handle = port.handle().clone_handle()?;

        Ok((
            Box::new(Device::new(port, val).register_handler(|_: MapTx, val| {
                let mappings = val
                    .bus
                    .lock()
                    .mapping_address(val.vendor, val.device)
                    .unwrap();

                Ok(MapRx {
                    data: mappings
                        .into_iter()
                        .map(|x| PciMapping {
                            base: x.start().bits() as _,
                            size: x.size() as _,
                        })
                        .collect(),
                })
            })),
            raw_handle,
        ))
    }
}
