use super::regs::rdesc::Rdesc;
use crate::e1000::regs::E1000Regs;
use alloc::vec::Vec;
use core::mem::size_of;
use dma::DmaBuffer;
use hal::address::{Address, PhysAddr};
use libc::irq::Irq;
use rtl::error::ErrorType;

// 16384 is the max size
const MAX_ORDER: usize = 13;
const MIN_ORDER: usize = 7;

pub struct RxBuffer {
    ring: DmaBuffer<Rdesc>,
    data: DmaBuffer<u8>,
    entry_order: u8,
    next_idx: u32,
    irq: Irq,
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
            ring,
            data,
            entry_order: entry_order as u8,
            next_idx: 0,
            irq,
        })
    }

    pub fn read_packet(&mut self, regs: &mut E1000Regs) -> Result<Vec<u8>, ErrorType> {
        let packet_size = 1 << self.data_order();
        let mut packet = Vec::with_capacity(packet_size);

        let index = self.next_idx as usize;
        let mut desc = self.ring.read(index);
        let num_descriptors = self.num_descriptors() as u32;

        while !desc.is_ready() {
            self.irq.wait()?;
            desc = self.ring.read(index);
        }

        // TODO: check errors and EOP

        let data = self
            .data
            .read_slice(index * (1 << self.entry_order), desc.length as usize);

        packet.extend_from_slice(data);

        // Clear DD flag
        desc.ack();
        self.ring.write(index, desc);

        // Update RDT after clearing DD
        regs.set_rdt(index as u32);

        self.next_idx = (self.next_idx + 1) % num_descriptors;
        Ok(packet)
    }

    pub fn num_descriptors(&self) -> usize {
        assert_eq!(self.ring.size() % size_of::<Rdesc>(), 0);

        return self.ring.size() / size_of::<Rdesc>();
    }

    pub fn data_order(&self) -> u8 {
        self.entry_order
    }

    pub fn ring_pa(&self) -> PhysAddr {
        self.ring.pa()
    }

    pub fn ring_size(&self) -> usize {
        self.ring.size()
    }
}
