use crate::bindings_Device::{Device, DoTx};
use alloc::boxed::Box;
use libc::handle::Handle;
use libc::port::Port;
use rtl::error::ErrorType;

pub struct PciDevice {
    vendor: u16,
    device: u16,
}

impl PciDevice {
    pub fn new(vendor: u16, device: u16) -> Result<(Box<Device<Self>>, Handle), ErrorType> {
        let port = Port::create()?;
        let val = Self { vendor, device };
        let raw_handle = port.handle().clone_handle()?;

        Ok((
            Box::new(Device::new(port, val).register_handler(|v: DoTx, _| todo!())),
            raw_handle,
        ))
    }
}

