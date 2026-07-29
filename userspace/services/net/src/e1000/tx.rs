use super::regs::Tdesc;
use dma::DmaBuffer;
use hal::address::{Address, PhysAddr};
use rtl::error::ErrorType;

const ONE_SLOT: usize = 1500;

pub struct TxBuffer {
    ring: DmaBuffer<Tdesc>,
    data: DmaBuffer<u8>,
}

impl TxBuffer {
    pub fn new(num_descriptors: usize) -> Result<Self, ErrorType> {
        let buffer_size = num_descriptors * ONE_SLOT;

        let mut ring = DmaBuffer::new(num_descriptors)?;
        let data = DmaBuffer::new(buffer_size)?;

        for i in 0..num_descriptors {
            ring.write(
                i,
                Tdesc {
                    buffer: (data.pa() + i * ONE_SLOT).bits() as u64,
                    ..Default::default()
                },
            );
        }

        Ok(Self {
            ring,
            data,
        })
    }

    pub fn ring_pa(&self) -> PhysAddr {
        self.ring.pa()
    }

    pub fn ring_size(&self) -> usize {
        self.ring.size()
    }
}
