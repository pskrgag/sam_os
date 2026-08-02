use super::frame::{EthFrame, EthFrameType, FrameType};
use super::mac::Mac;
use crate::ip::v4::IPv4;
use rtl::error::ErrorType;
use zerocopy::FromBytes;

const ARP_PACKET_SIZE: usize = 28;

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

/// Address resolution protocol frame
#[derive(Debug)]
pub struct Arp {
    hardware_type: ArpHardware,
    protocol_type: ArpProtocol,
    hw_address_length: u8,
    protocol_length: u8,
    operation: ArpOperation,
    sender_hw: Mac,
    sender_ip: IPv4,
    target_hw: Mac,
    target_ip: IPv4,
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

impl Arp {
    pub fn new(
        operation: ArpOperation,
        sender_hw: Mac,
        sender_ip: IPv4,
        target_hw: Mac,
        target_ip: IPv4,
    ) -> Self {
        Self {
            hardware_type: ArpHardware::Ethernet,
            protocol_type: ArpProtocol::IPv4,
            hw_address_length: 6,
            protocol_length: 4,
            operation,
            sender_hw,
            sender_ip,
            target_hw,
            target_ip,
        }
    }

    pub fn target_mac(&self) -> Mac {
        self.target_hw
    }

    pub fn target_ip(&self) -> IPv4 {
        self.target_ip
    }

    pub fn sender_mac(&self) -> Mac {
        self.sender_hw
    }

    pub fn sender_ip(&self) -> IPv4 {
        self.sender_ip
    }
}

impl<'a> TryFrom<&'a [u8]> for Arp {
    type Error = ErrorType;

    fn try_from(frame: &'a [u8]) -> Result<Self, Self::Error> {
        let (hardware_type, remaining) =
            <u16>::read_from_prefix(frame).map_err(|_| ErrorType::BufferTooSmall)?;

        let (protocol_type, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (hw_address_length, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (protocol_length, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (operation, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;

        if hw_address_length != 6 || protocol_length != 4 {
            return Err(ErrorType::InvalidArgument);
        }

        let operation = match u16::from_be(operation) {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            _ => return Err(ErrorType::InvalidArgument),
        };

        let (sender_hw, remaining) =
            <[u8; 6]>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (sender_ip, remaining) =
            <[u8; 4]>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (target_hw, remaining) =
            <[u8; 6]>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (target_ip, _) =
            <[u8; 4]>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;

        Ok(Self {
            hardware_type: u16::from_be(hardware_type).try_into()?,
            protocol_type: u16::from_be(protocol_type).try_into()?,
            hw_address_length,
            protocol_length,
            operation,
            sender_hw: sender_hw.into(),
            sender_ip: sender_ip.into(),
            target_hw: target_hw.into(),
            target_ip: target_ip.into(),
        })
    }
}

impl EthFrameType for Arp {
    const TYPE: FrameType = FrameType::ARP;
}

impl EthFrame for Arp {
    fn serialize(&self, output: &mut [u8]) -> Result<usize, ErrorType> {
        if output.len() < ARP_PACKET_SIZE {
            return Err(ErrorType::BufferTooSmall);
        }

        output[0..2].copy_from_slice(&(self.hardware_type as u16).to_be_bytes());
        output[2..4].copy_from_slice(&(self.protocol_type as u16).to_be_bytes());
        output[4] = self.hw_address_length;
        output[5] = self.protocol_length;
        output[6..8].copy_from_slice(&(self.operation as u16).to_be_bytes());

        let sender_hw = u64::from(self.sender_hw).to_ne_bytes();
        output[8..14].copy_from_slice(&sender_hw[..6]);
        output[14..18].copy_from_slice(self.sender_ip.as_slice());

        let target_hw = u64::from(self.target_hw).to_ne_bytes();
        output[18..24].copy_from_slice(&target_hw[..6]);
        output[24..28].copy_from_slice(self.target_ip.as_slice());

        Ok(ARP_PACKET_SIZE)
    }

    fn serialize_len(&self) -> usize {
        ARP_PACKET_SIZE
    }
}
