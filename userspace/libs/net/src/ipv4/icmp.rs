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
}

#[repr(C)]
#[derive(Debug, FromBytes, Immutable, KnownLayout, Unaligned, IntoBytes)]
pub struct IcmpEchoHeader {
    identifier: [u8; 2],
    sequence: [u8; 2],
}

impl IcmpHeader {
    pub fn new(packet_type: u8, code: u8) -> Self {
        Self {
            packet_type,
            code,
            checksum: [0; 2],
        }
    }

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
}

impl IcmpEchoHeader {
    pub fn new(identifier: u16, sequence: u16) -> Self {
        Self {
            identifier: identifier.to_be_bytes(),
            sequence: sequence.to_be_bytes(),
        }
    }
}

impl Header for IcmpEchoHeader {
    type Error = ErrorType;

    fn header_len(data: &[u8]) -> Result<usize, Self::Error> {
        Self::ref_from_prefix(data).map_err(|_| ErrorType::BufferTooSmall)?;

        Ok(Self::fixed_len())
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
