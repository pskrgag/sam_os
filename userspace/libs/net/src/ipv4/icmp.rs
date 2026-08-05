use crate::{checksum::checksum, header::Header};
use rtl::error::ErrorType;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(Debug)]
#[repr(u8)]
pub enum PacketType {
    EchoRequest = 8,
    EchoReply = 0,
}

#[repr(C)]
#[derive(Debug, FromBytes, Immutable, KnownLayout, Unaligned, IntoBytes)]
pub struct IcmpHeader {
    packet_type: u8,
    code: u8,
    checksum: [u8; 2],
    rest: [u8; 4],
}

impl IcmpHeader {
    pub fn packet_type(&self) -> Result<PacketType, ErrorType> {
        match self.packet_type {
            x if x == PacketType::EchoRequest as u8 => Ok(PacketType::EchoRequest),
            x if x == PacketType::EchoReply as u8 => Ok(PacketType::EchoReply),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
    pub fn set_packet_type(&mut self, tp: PacketType) {
        self.packet_type = tp as _;
    }
    pub fn set_checksum(&mut self, checksum: u16) {
        self.checksum = checksum.to_be_bytes();
    }
    pub fn code(&self) -> u8 {
        self.code
    }
    pub fn echo_id(&self) -> u16 {
        u16::from_be_bytes([self.rest[0], self.rest[1]])
    }
    pub fn echo_sequence(&self) -> u16 {
        u16::from_be_bytes([self.rest[2], self.rest[3]])
    }
}

impl Header for IcmpHeader {
    type Error = ErrorType;
    fn header_len(data: &[u8]) -> Result<usize, Self::Error> {
        Self::ref_from_prefix(data).map_err(|_| ErrorType::BufferTooSmall)?;
        if checksum(data) != 0 {
            return Err(ErrorType::InvalidArgument);
        }
        Ok(Self::fixed_len())
    }
}
