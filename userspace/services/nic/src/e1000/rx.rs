use super::regs::rdesc::Rdesc;
use crate::e1000::regs::E1000Regs;
use alloc::vec::Vec;
use core::mem::size_of;
use dma::DmaBuffer;
use hal::address::{Address, PhysAddr};
use rokio::irq::Irq;
use rtl::error::ErrorType;
use spin::Mutex;

// 16384 is the max size
const MAX_ORDER: usize = 13;
const MIN_ORDER: usize = 7;

struct RxBufferInner {
    ring: DmaBuffer<Rdesc>,
    data: DmaBuffer<u8>,
    next_idx: u32,
}

pub struct RxBuffer {
    entry_order: u8,
    irq: Irq,
    inner: Mutex<RxBufferInner>,
}

impl RxBufferInner {
    fn num_descriptors(&self) -> usize {
        assert_eq!(self.ring.size() % size_of::<Rdesc>(), 0);

        return self.ring.size() / size_of::<Rdesc>();
    }
}

impl RxBuffer {
    pub fn new(num_descriptors: usize, entry_order: usize, irq: Irq) -> Result<Self, ErrorType> {
        if !(MIN_ORDER..=MAX_ORDER).contains(&entry_order) {
            return Err(ErrorType::InvalidArgument);
        }

        let entry_size = 1 << entry_order;
        let buffer_size = num_descriptors * entry_size;

        let mut ring = DmaBuffer::new(num_descriptors)?;
        let data = DmaBuffer::new(buffer_size)?;

        for i in 0..num_descriptors {
            ring.write(
                i,
                Rdesc {
                    buffer: (data.pa() + i * entry_size).bits() as u64,
                    ..Default::default()
                },
            );
        }

        Ok(Self {
            inner: Mutex::new(RxBufferInner {
                ring,
                data,
                next_idx: 0,
            }),
            entry_order: entry_order as u8,
            irq,
        })
    }

    pub async fn read_packet(&self, regs: &Mutex<E1000Regs>) -> Result<Vec<u8>, ErrorType> {
        let packet_size = 1 << self.data_order();
        let mut packet = Vec::with_capacity(packet_size);

        // TODO: we need to check for idx overflow

        let (mut desc, index) = {
            let mut inner = self.inner.lock();
            let index = inner.next_idx as usize;
            let num_descriptors = inner.num_descriptors() as u32;

            // We are going to unlock the lock while waiting for the IRQ. Other threads must observe
            // different idx.
            inner.next_idx = (inner.next_idx + 1) % num_descriptors;

            (inner.ring.read(index), index)
        };

        while !desc.is_ready() {
            println!("Waiting for the IRQ");
            self.irq.wait().await?;

            let mut inner = self.inner.lock();
            desc = inner.ring.read(index);
        }

        println!("Received packet");
        let mut inner = self.inner.lock();
        let mut regs = regs.lock();

        regs.ack_irq();
        self.irq.ack();

        // TODO: check errors and EOP

        let data = inner
            .data
            .read_slice(index * (1 << self.entry_order), desc.length as usize);

        packet.extend_from_slice(data);

        // Clear DD flag
        desc.ack();
        inner.ring.write(index, desc);

        // Update RDT after clearing DD
        regs.set_rdt(index as u32);
        Ok(packet)
    }

    // pub fn num_descriptors(&self) -> usize {
    //     self.inner.lock().num_descriptors()
    // }

    pub fn data_order(&self) -> u8 {
        self.entry_order
    }

    pub fn ring_pa(&self) -> PhysAddr {
        let inner = self.inner.lock();

        inner.ring.pa()
    }

    pub fn ring_size(&self) -> usize {
        let inner = self.inner.lock();

        inner.ring.size()
    }
}
