use crate::vmm::types::VirtAddr;

#[cfg(target_arch = "aarch64")]
pub mod arm_uart;
#[cfg(target_arch = "aarch64")]
pub use arm_uart::*;

pub trait UartTrait {
    fn write_bytes(&mut self, bytes: &[u8]);
    fn read_bytes(&mut self, bytes: &mut [u8]);
    fn base(&self) -> VirtAddr;
}
