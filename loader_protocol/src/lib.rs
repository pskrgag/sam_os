#![no_std]

use heapless::Vec;
use rtl::vmm::types::VirtAddr;

pub const MAX_DEVICES: usize = 10;

#[derive(Debug, PartialEq)]
pub enum DeviceKind {
    Uart,
}

#[derive(Debug)]
pub struct DeviceMapping {
    pub base: usize,
    pub size: usize,
    pub kind: DeviceKind,
}

#[derive(Default)]
pub struct LoaderArg {
    pub tt_base: usize,
    pub fdt_base: usize,
    pub devices: Vec<DeviceMapping, MAX_DEVICES>,
}

impl LoaderArg {
    pub fn get_device(&self, kind: DeviceKind) -> Option<(VirtAddr, usize)> {
        self.devices
            .iter()
            .find(|x| x.kind == kind)
            .map(|x| (x.base.into(), x.size))
    }
}
