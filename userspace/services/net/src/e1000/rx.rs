use super::regs::rdesc::Rdesc;
use dma::DmaBuffer;
use hal::address::{Address, PhysAddr};
use rtl::error::ErrorType;

// 16384 is the max size
const MAX_ORDER: usize = 13;
const MIN_ORDER: usize = 7;

pub struct RxBuffer {
    ring: DmaBuffer<Rdesc>,
    data: DmaBuffer<u8>,
    entry_order: u8,
}

impl RxBuffer {
    pub fn new(num_descriptors: usize, entry_order: usize) -> Result<Self, ErrorType> {
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
        })
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
