use super::Mac;
use crate::{header::Header, ipv4::IPv4};
use rtl::error::ErrorType;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum ArpHardware {
    Ethernet = 1,
}

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum ArpProtocol {
    IPv4 = 0x800,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
}

#[repr(C)]
#[derive(Debug, FromBytes, Immutable, KnownLayout, Unaligned, IntoBytes)]
pub struct ArpHeader {
    hardware_type: [u8; 2],
    protocol_type: [u8; 2],
    hw_address_length: u8,
    protocol_length: u8,
    operation: [u8; 2],
}

#[repr(C)]
#[derive(Debug, FromBytes, Immutable, KnownLayout, Unaligned, IntoBytes)]
pub struct ArpPayload {
    pub sender_mac: Mac,
    pub sender_ip: IPv4,
    pub target_mac: Mac,
    pub target_ip: IPv4,
}

impl ArpHeader {
    pub fn hardware_type(&self) -> Result<ArpHardware, ErrorType> {
        u16::from_be_bytes(self.hardware_type).try_into()
    }
    pub fn protocol_type(&self) -> Result<ArpProtocol, ErrorType> {
        u16::from_be_bytes(self.protocol_type).try_into()
    }
    pub fn set_operation(&mut self, op: ArpOperation) {
        self.operation = (op as u16).to_be_bytes();
    }
    pub fn operation(&self) -> Result<ArpOperation, ErrorType> {
        match u16::from_be_bytes(self.operation) {
            1 => Ok(ArpOperation::Request),
            2 => Ok(ArpOperation::Reply),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
    pub fn hardware_address_len(&self) -> usize {
        self.hw_address_length as usize
    }
    pub fn protocol_address_len(&self) -> usize {
        self.protocol_length as usize
    }
}

impl TryFrom<u16> for ArpHardware {
    type Error = ErrorType;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::Ethernet as u16 => Ok(Self::Ethernet),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
}

impl TryFrom<u16> for ArpProtocol {
    type Error = ErrorType;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::IPv4 as u16 => Ok(Self::IPv4),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
}

impl Header for ArpHeader {
    type Error = ErrorType;
    fn header_len(data: &[u8]) -> Result<usize, Self::Error> {
        let header = Self::ref_from_prefix(data)
            .map(|(header, _)| header)
            .map_err(|_| ErrorType::BufferTooSmall)?;
        header.hardware_type()?;
        header.protocol_type()?;
        header.operation()?;
        let addresses_len = 2 * (header.hardware_address_len() + header.protocol_address_len());
        if data.len() < Self::fixed_len() + addresses_len {
            return Err(ErrorType::BufferTooSmall);
        }
        Ok(Self::fixed_len())
    }
}
