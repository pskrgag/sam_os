use crate::bindings_NameServer::NameServer;
use crate::bindings_Pci::{Device, DeviceId, Pci};
use crate::driver::Nic;
use alloc::boxed::Box;
use alloc::vec::Vec;
use hal::address::MemRange;
use hal::arch::PAGE_SIZE;
use libc::vmm::vms::vms;
use net::ethernet::Mac;
use regs::E1000Error;
use regs::E1000Regs;
use rokio::irq::Irq;
use rokio::port::Port;
use rtl::error::ErrorType;
use rx::RxBuffer;
use spin::Mutex;
use tx::TxBuffer;

mod regs;
mod rx;
mod tx;

pub struct E1000 {
    device: Device,
    mac: Mac,
    regs: Mutex<E1000Regs>,
    tx_buffer: TxBuffer,
    rx_buffer: RxBuffer,
}

impl E1000 {
    pub async fn new(ns: &NameServer) -> Result<Self, E1000Error> {
        let pci = ns.Get("pci".try_into().unwrap()).await.unwrap();
        let pci = unsafe { Pci::new(Port::new(pci.handle)) };

        // These IDS are from QEMU
        let bfds = pci
            .Find(DeviceId {
                vendor: 0x8086,
                device: 0x100e,
            })
            .await
            .unwrap();

        let device = Device::new(unsafe {
            Port::new(pci.Open(bfds.addresses[0].clone()).await.unwrap().device)
        });

        let irq = unsafe { Irq::new_from_handle(device.AllocateIrq().await?.irq) };

        let res = device.Map().await.unwrap();
        assert_eq!(res.data.len(), 1);

        let mmio = vms()
            .map_phys(MemRange::new(
                (res.data[0].base as usize).into(),
                (res.data[0].size as usize).next_multiple_of(PAGE_SIZE),
            ))
            .unwrap();

        let rx_buffer = RxBuffer::new(1000, 11, irq).map_err(|_| E1000Error::NoMemory)?;
        let tx_buffer = TxBuffer::new(1000).map_err(|_| E1000Error::NoMemory)?;

        let mut regs = E1000Regs::new(mmio, &tx_buffer, &rx_buffer)?;
        let mac = regs.mac()?;
        let mac = mac.try_into().expect("Invalid MAC? Don't think so...");

        println!("Mac {:?}", mac);

        Ok(Self {
            device,
            regs: Mutex::new(regs),
            mac,
            rx_buffer,
            tx_buffer,
        })
    }

    pub fn send_packet(&self, data: &[u8]) {
        self.tx_buffer.send_packet(data, &self.regs)
    }

    pub async fn read_packet(&self) -> Result<Vec<u8>, ErrorType> {
        self.rx_buffer.read_packet(&self.regs).await
    }
}

#[async_trait::async_trait]
impl Nic for E1000 {
    async fn receive_frame(&self) -> Result<Vec<u8>, ErrorType> {
        self.read_packet().await
    }

    fn send_frame(&self, data: &[u8]) -> Result<(), ErrorType> {
        self.send_packet(data);
        Ok(())
    }

    fn mac(&self) -> Mac {
        self.mac
    }
}
