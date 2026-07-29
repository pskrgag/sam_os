use super::regs::Tdesc;
use crate::e1000::regs::E1000Regs;
use dma::DmaBuffer;
use hal::address::{Address, PhysAddr};
use rtl::error::ErrorType;

const ONE_SLOT: usize = 1500;

pub struct TxBuffer {
    ring: DmaBuffer<Tdesc>,
    data: DmaBuffer<u8>,
    next_idx: usize,
}

impl TxBuffer {
    pub fn new(num_descriptors: usize) -> Result<Self, ErrorType> {
        let buffer_size = num_descriptors * ONE_SLOT;

        let ring = DmaBuffer::new(num_descriptors)?;
        let data = DmaBuffer::new(buffer_size)?;

        Ok(Self {
            ring,
            data,
            next_idx: 0,
        })
    }

    pub fn send_packet(&mut self, data: &[u8], regs: &mut E1000Regs) {
        let slot = self.next_idx;

        assert!(data.len() <= ONE_SLOT);
        self.data.write_slice(slot * ONE_SLOT, data);
        self.ring.write(
            slot,
            Tdesc::new(
                (self.data.pa() + slot * ONE_SLOT).bits() as u64,
                data.len() as u16,
            ),
        );

        regs.set_tdt(slot as u32 + 1);
        while !self.ring.read(slot).is_ready() {}

        self.next_idx = (self.next_idx + 1) % self.num_descriptors();
    }

    pub fn num_descriptors(&self) -> usize {
        assert_eq!(self.ring.size() % size_of::<Tdesc>(), 0);

        return self.ring.size() / size_of::<Tdesc>();
    }

    pub fn ring_pa(&self) -> PhysAddr {
        self.ring.pa()
    }

    pub fn ring_size(&self) -> usize {
        self.ring.size()
    }
}
