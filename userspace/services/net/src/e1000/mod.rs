use crate::bindings_NameServer::NameServer;
use crate::bindings_Pci::{Device, Pci};
use crate::net::eth::mac::Mac;
use hal::address::MemRange;
use hal::arch::PAGE_SIZE;
use libc::vmm::vms::vms;
use regs::E1000Regs;
use rokio::port::Port;
use rtl::error::ErrorType;
use regs::E1000Error;

mod regs;

pub struct E1000 {
    device: Device,
    mac: Mac,
    regs: E1000Regs,
}

impl E1000 {
    pub async fn new(ns: NameServer) -> Result<Self, E1000Error> {
        let pci = ns.Get("pci".try_into().unwrap()).await.unwrap();
        let pci = unsafe { Pci::new(Port::new(pci.handle)) };

        // These IDS are from QEMU
        let device =
            Device::new(unsafe { Port::new(pci.Device(0x8086, 0x100e).await.unwrap().handle) });

        let res = device.Map().await.unwrap();
        assert_eq!(res.data.len(), 1);

        let va = vms()
            .map_phys(MemRange::new(
                (res.data[0].base as usize).into(),
                (res.data[0].size as usize).next_multiple_of(PAGE_SIZE),
            ))
            .unwrap();

        let mut regs = E1000Regs::new(va)?;
        let mac = regs.mac()?;
        let mac = Mac::from_raw(mac).expect("Invalid MAC? Don't think so...");

        println!("Mac {:?}", mac);

        Ok(Self { device, regs, mac })
    }
}
