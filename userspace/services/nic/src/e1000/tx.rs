use super::regs::Tdesc;
use crate::e1000::regs::E1000Regs;
use dma::DmaBuffer;
use hal::address::{Address, PhysAddr};
use rtl::error::ErrorType;
use spin::Mutex;

const ONE_SLOT: usize = 1500;

struct TxBufferInner {
    ring: DmaBuffer<Tdesc>,
    data: DmaBuffer<u8>,
    next_idx: usize,
}

pub struct TxBuffer {
    inner: Mutex<TxBufferInner>,
}

impl TxBuffer {
    pub fn new(num_descriptors: usize) -> Result<Self, ErrorType> {
        let buffer_size = num_descriptors * ONE_SLOT;

        let ring = DmaBuffer::new(num_descriptors)?;
        let data = DmaBuffer::new(buffer_size)?;

        Ok(Self {
            inner: Mutex::new(TxBufferInner {
                ring,
                data,
                next_idx: 0,
            }),
        })
    }

    pub fn send_packet(&self, data: &[u8], regs: &Mutex<E1000Regs>) {
        let mut inner = self.inner.lock();
        let slot = inner.next_idx;

        assert!(data.len() <= ONE_SLOT);

        inner.data.write_slice(slot * ONE_SLOT, data);
        let buffer = (inner.data.pa() + slot * ONE_SLOT).bits() as u64;

        inner.ring.write(
            slot,
            Tdesc::new(buffer, data.len() as u16),
        );

        regs.lock().set_tdt(slot as u32 + 1);
        while !inner.ring.read(slot).is_ready() {}

        let num_descriptors = inner.ring.size() / size_of::<Tdesc>();
        inner.next_idx = (inner.next_idx + 1) % num_descriptors;
    }

    pub fn num_descriptors(&self) -> usize {
        let inner = self.inner.lock();
        assert_eq!(inner.ring.size() % size_of::<Tdesc>(), 0);

        inner.ring.size() / size_of::<Tdesc>()
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
