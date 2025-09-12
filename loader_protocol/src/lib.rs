#![no_std]

use heapless::Vec;
use rtl::vmm::types::VirtAddr;

pub const MAX_DEVICES: usize = 10;
pub const MAX_VMM_REGIONS: usize = 10;

#[derive(Debug, PartialEq)]
pub enum DeviceKind {
    Uart,
    GicDist,
    GicCpu,
}

#[derive(Debug, PartialEq, Clone)]
#[repr(usize)]
pub enum VmmLayoutKind {
    Image,
    Mmio,
    Fixmap,
    LoaderArg,
    VmAlloc,
}

#[derive(Debug)]
pub struct DeviceMapping {
    pub base: usize,
    pub size: usize,
    pub kind: DeviceKind,
}

#[derive(Debug, Clone)]
pub struct VmmLayoutEntry {
    pub base: usize,
    pub size: usize,
    pub kind: VmmLayoutKind,
}

#[derive(Default)]
pub struct LoaderArg {
    pub tt_base: usize,
    pub fdt_base: usize,
    pub devices: Vec<DeviceMapping, MAX_DEVICES>,
    pub vmm_layout: Vec<VmmLayoutEntry, MAX_VMM_REGIONS>,
}

impl LoaderArg {
    pub fn get_device(&self, kind: DeviceKind) -> Option<(VirtAddr, usize)> {
        self.devices
            .iter()
            .find(|x| x.kind == kind)
            .map(|x| (x.base.into(), x.size))
    }

    pub fn get_vmm_base(&self, kind: VmmLayoutKind) -> Option<(VirtAddr, usize)> {
        self.vmm_layout
            .iter()
            .find(|x| x.kind == kind)
            .map(|x| (x.base.into(), x.size))
    }
}
