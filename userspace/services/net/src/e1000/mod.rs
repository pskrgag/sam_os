use crate::bindings_NameServer::NameServer;
use crate::bindings_Pci::{Device, Pci};
use crate::net::eth::mac::Mac;
use alloc::vec::Vec;
use hal::address::MemRange;
use hal::arch::PAGE_SIZE;
use libc::vmm::vms::vms;
use regs::E1000Error;
use regs::E1000Regs;
use rokio::port::Port;
use rx::RxBuffer;
use tx::TxBuffer;

mod regs;
mod rx;
mod tx;

pub struct E1000 {
    device: Device,
    mac: Mac,
    regs: E1000Regs,
    tx_buffer: TxBuffer,
    rx_buffer: RxBuffer,
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

        let mmio = vms()
            .map_phys(MemRange::new(
                (res.data[0].base as usize).into(),
                (res.data[0].size as usize).next_multiple_of(PAGE_SIZE),
            ))
            .unwrap();

        let rx_buffer = RxBuffer::new(1000, 11).map_err(|_| E1000Error::NoMemory)?;
        let tx_buffer = TxBuffer::new(1000).map_err(|_| E1000Error::NoMemory)?;

        let mut regs = E1000Regs::new(mmio, &tx_buffer, &rx_buffer)?;
        let mac = regs.mac()?;
        let mac = Mac::from_raw(mac).expect("Invalid MAC? Don't think so...");

        println!("Mac {:?}", mac);

        Ok(Self {
            device,
            regs,
            mac,
            rx_buffer,
            tx_buffer,
        })
    }

    pub fn send_packet(&mut self, data: &[u8]) {
        self.tx_buffer.send_packet(data, &mut self.regs)
    }

    pub fn read_packet(&mut self) -> Option<Vec<u8>> {
        self.rx_buffer.read_packet(&mut self.regs)
    }
}
