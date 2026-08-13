use alloc::boxed::Box;
use alloc::vec::Vec;
use net::ethernet::Mac;
use rtl::error::ErrorType;

#[async_trait::async_trait]
pub trait Nic: Send + Sync {
    // Receives one frame to the NIC
    async fn receive_frame(&self) -> Result<Vec<u8>, ErrorType>;

    // Sends one frame to the NIC
    fn send_frame(&self, data: &[u8]) -> Result<(), ErrorType>;

    // NIC's MAC
    fn mac(&self) -> Mac;
}
