pub mod uart;

pub trait Backend {
    fn read_byte(&self) -> Option<u8>;
    fn write_byte(&self, b: u8) -> Option<()>;
    fn write_bytes(&self, b: &[u8]) -> Option<()>;
}
