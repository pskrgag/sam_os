use super::sdhci::SdhciIface;
use alloc::vec::Vec;
use core::ops::Deref;
use rtl::error::ErrorType;

pub mod sd;

pub trait Card: Send + Deref<Target = SdhciIface> {
    fn read_block(&mut self, block: u32) -> Result<Vec<u8>, ErrorType>;
    fn block_size(&self) -> u16;
    fn device_size(&self) -> usize;
}
