mod address;
pub mod icmp;

pub use address::{IPv4, Ipv4Config};

use crate::{checksum::checksum, header::Header};
use rtl::error::ErrorType;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Protocol {
    ICMP = 0x1,
}

impl TryFrom<u8> for Protocol {
    type Error = ErrorType;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::ICMP as u8 => Ok(Self::ICMP),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, Immutable, KnownLayout, Unaligned, IntoBytes, Copy, Clone)]
pub struct IPv4Header {
    version_ihl: u8,
    ds_fields: u8,
    total_length: [u8; 2],
    id: [u8; 2],
    flags_fragment_offset: [u8; 2],
    ttl: u8,
    protocol: u8,
    checksum: [u8; 2],
    src: [u8; 4],
    dst: [u8; 4],
}

impl IPv4Header {
    pub fn new(dst: IPv4, src: IPv4, protocol: Protocol, payload_size: u16) -> Self {
        let mut res = Self {
            version_ihl: (4 << 4) | Self::fixed_len() as u8 / 4,
            ds_fields: 0,
            total_length: (payload_size + Self::fixed_len() as u16).to_be_bytes(),
            id: [0; 2],
            flags_fragment_offset: [0; 2],
            ttl: 64,
            protocol: protocol as _,
            checksum: [0; 2],
            src: src.into(),
            dst: dst.into(),
        };

        res.checksum();
        res
    }

    pub fn header_length(&self) -> usize {
        (self.version_ihl as usize & 0x0f) * 4
    }

    pub fn total_length(&self) -> usize {
        u16::from_be_bytes(self.total_length) as usize
    }

    pub fn swap_ips(&mut self) {
        core::mem::swap(&mut self.dst, &mut self.src);
    }

    pub fn protocol(&self) -> Result<Protocol, ErrorType> {
        self.protocol.try_into()
    }

    pub fn source(&self) -> IPv4 {
        self.src.into()
    }

    pub fn destination(&self) -> IPv4 {
        self.dst.into()
    }

    pub fn ttl(&self) -> u8 {
        self.ttl
    }

    pub fn checksum(&mut self) {
        self.checksum = [0; 2];
        self.checksum = checksum(self.as_bytes()).to_be_bytes();
    }
}

impl Header for IPv4Header {
    type Error = ErrorType;

    fn header_len(data: &[u8]) -> Result<usize, Self::Error> {
        let header = Self::ref_from_prefix(data)
            .map(|(header, _)| header)
            .map_err(|_| ErrorType::BufferTooSmall)?;

        if header.version_ihl >> 4 != 4 || header.header_length() < Self::fixed_len() {
            return Err(ErrorType::InvalidArgument);
        }

        if data.len() < header.total_length() || header.total_length() < header.header_length() {
            return Err(ErrorType::BufferTooSmall);
        }

        if checksum(&data[..header.header_length()]) != 0 {
            return Err(ErrorType::InvalidArgument);
        }

        header.protocol()?;
        Ok(header.header_length())
    }
}
