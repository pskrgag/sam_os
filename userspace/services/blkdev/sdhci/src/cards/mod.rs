use super::sdhci::SdhciIface;
use alloc::vec::Vec;
use core::ops::Deref;
use rtl::error::ErrorType;

pub mod sd;

pub trait Card: Send + Deref<Target = SdhciIface> {
    /// Reads one block of data from the device
    fn read_block(&mut self, block: u32) -> Result<Vec<u8>, ErrorType>;

    /// Returns current block count
    fn block_size(&self) -> u16;

    /// Total size of the device in bytes
    fn device_size(&self) -> usize;

    /// Sets block size for the device
    fn set_block_size(&mut self, block_size: u16) -> Result<(), ErrorType>;
}
