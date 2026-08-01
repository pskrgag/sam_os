use super::ecam::{PciEcam, PciIrqPin};
use crate::bindings_Pci::{Device, DeviceRequest, PciMapping};
use alloc::sync::Arc;
use core::future::Future;
use hal::address::Address;
use heapless::Vec;
use libc::handle::Handle;
use rokio::port::Port;
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
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        let port = Port::create()?;
        let device = Arc::new(Spinlock::new(Self {
            vendor,
            device,
            bus,
        }));
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
                                .mapping_address(device.vendor, device.device)
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
                        DeviceRequest::AllocateIrq { responder, value } => {
                            let device = device.lock();
                            let irq = device
                                .bus
                                .lock()
                                .allocate_irq(device.vendor, device.device)?;

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
