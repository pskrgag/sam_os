use alloc::vec::Vec;
use rtl::error::ErrorType;
use net::eth::mac::Mac;

pub trait Nic: Send {
    // Receives one frame to the NIC
    fn receive_frame(&mut self) -> Result<Vec<u8>, ErrorType>;

    // Sends one frame to the NIC
    fn send_frame(&mut self, data: &[u8]) -> Result<(), ErrorType>;

    // NIC's MAC
    fn mac(&self) -> Mac;
}
