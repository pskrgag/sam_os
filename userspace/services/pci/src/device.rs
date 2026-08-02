use super::ecam::{DeviceInfo, PciEcam};
use crate::bindings_Pci::{Device, DeviceRequest, PciMapping};
use alloc::sync::Arc;
use core::future::Future;
use hal::address::Address;
use heapless::Vec;
use libc::handle::Handle;
use pci_types::PciAddress;
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub struct PciDevice {
    address: PciAddress,
    bus: Arc<Spinlock<PciEcam>>,
    info: DeviceInfo,
}

impl PciDevice {
    pub fn new(
        address: PciAddress,
        bus: Arc<Spinlock<PciEcam>>,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        let info = bus
            .lock()
            .device_info(address)
            .ok_or(ErrorType::InvalidArgument)?;
        let port = Port::create()?;
        let device = Arc::new(Spinlock::new(Self { address, bus, info }));
        let raw_handle = port.handle().clone_handle()?;

        Ok((
            Device::for_each(port, move |req| {
                let device = device.clone();

                async move {
                    match req {
                        DeviceRequest::Map { responder, .. } => {
                            let device = device.lock();

                            let mappings: Vec<PciMapping, 6> = device
                                .bus
                                .lock()
                                .mapping_address(device.address)
                                .unwrap()
                                .into_iter()
                                .map(|x| PciMapping {
                                    base: x.range.start().bits() as _,
                                    size: x.range.size() as _,
                                    index: x.index,
                                })
                                .collect();

                            responder.reply(mappings)?;
                        }
                        DeviceRequest::AllocateIrq { responder, .. } => {
                            let device = device.lock();
                            let irq = device.bus.lock().allocate_irq(device.address)?;

                            responder.reply(irq.handle())?;
                        }
                    }

                    Ok(())
                }
            }),
            raw_handle,
        ))
    }
}
