use super::*;
use interfaces::implementation::serial;

pub struct UartBackend {}

impl Backend for UartBackend {
    fn read_byte(&self) -> Option<u8> {
        serial::read_byte().ok()
    }

    fn write_byte(&self, b: u8) -> Option<()> {
        serial::write_byte(b).ok()
    }

    fn write_bytes(&self, b: &[u8]) -> Option<()> {
        serial::write_bytes(b).ok()
    }
}

impl Default for UartBackend {
    fn default() -> Self {
        UartBackend {}
    }
}
