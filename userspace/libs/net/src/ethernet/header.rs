use super::Mac;
use crate::header::Header;
use rtl::error::ErrorType;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum FrameType {
    ARP = 0x0806,
    IPv4 = 0x0800,
    IPv6 = 0x86DD,
}

impl TryFrom<u16> for FrameType {
    type Error = ErrorType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::ARP as u16 => Ok(Self::ARP),
            _ if value == Self::IPv6 as u16 => Ok(Self::IPv6),
            _ if value == Self::IPv4 as u16 => Ok(Self::IPv4),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
}

#[repr(C)]
#[derive(FromBytes, Immutable, KnownLayout, Unaligned, IntoBytes)]
pub struct EthHeader {
    dst: [u8; 6],
    src: [u8; 6],
    tp: [u8; 2],
}

impl EthHeader {
    pub fn new(dst: Mac, src: Mac, protocol: FrameType) -> Self {
        Self {
            dst: dst.into(),
            src: src.into(),
            tp: (protocol as u16).to_be_bytes(),
        }
    }

    pub fn destination(&self) -> Mac {
        self.dst.into()
    }

    pub fn source(&self) -> Mac {
        self.src.into()
    }

    pub fn swap_macs(&mut self) {
        core::mem::swap(&mut self.dst, &mut self.src);
    }

    pub fn set_source(&mut self, mac: Mac) {
        self.src = mac.into();
    }

    pub fn set_destination(&mut self, mac: Mac) {
        self.dst = mac.into();
    }

    pub fn frame_type(&self) -> Result<FrameType, ErrorType> {
        FrameType::try_from(u16::from_be_bytes(self.tp))
    }
}

impl Header for EthHeader {
    type Error = ErrorType;

    fn header_len(data: &[u8]) -> Result<usize, Self::Error> {
        let header = Self::ref_from_prefix(data)
            .map(|(header, _)| header)
            .map_err(|_| ErrorType::BufferTooSmall)?;
        header.frame_type()?;
        Ok(Self::fixed_len())
    }
}
