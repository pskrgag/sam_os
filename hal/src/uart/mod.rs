use crate::address::VirtAddr;

#[cfg(target_arch = "aarch64")]
pub mod pl011;
#[cfg(target_arch = "aarch64")]
pub use pl011::*;

pub trait UartTrait {
    fn write_bytes(&mut self, bytes: &[u8]);
    fn read_bytes(&mut self, bytes: &mut [u8]);
    fn base(&self) -> VirtAddr;
}
