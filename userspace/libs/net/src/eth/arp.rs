use super::frame::{EthFrame, FrameType};
use super::mac::Mac;
use rtl::error::ErrorType;
use zerocopy::FromBytes;

const ARP_PACKET_SIZE: usize = 28;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
}

/// Address resolution protocol frame
#[derive(Debug)]
pub struct Arp {
    hardware_type: u16,
    protocol_type: u16,
    hw_address_length: u8,
    protocol_length: u8,
    operation: ArpOperation,
    sender_hw: Mac,
    sender_ip: [u8; 4], // TODO: make ip
    target_hw: Mac,
    target_ip: [u8; 4], // TODO: make ip
}

impl Arp {
    pub fn new(
        op: ArpOperation,
        sender_hw: Mac,
        sender_ip: [u8; 4],
        target_hw: Mac,
        target_ip: [u8; 4],
    ) -> Self {
        Self {
            hardware_type: 1,
            protocol_type: 2048,
            hw_address_length: 6,
            protocol_length: 4,
            operation: op,
            sender_hw,
            sender_ip,
            target_hw,
            target_ip,
        }
    }

    pub fn target_mac(&self) -> Mac {
        self.target_hw
    }

    pub fn target_ip(&self) -> [u8; 4] {
        self.target_ip
    }

    pub fn sender_mac(&self) -> Mac {
        self.sender_hw
    }

    pub fn sender_ip(&self) -> [u8; 4] {
        self.sender_ip
    }
}

impl<'a> TryFrom<&'a [u8]> for Arp {
    type Error = ErrorType;

    fn try_from(frame: &'a [u8]) -> Result<Self, Self::Error> {
        let (hardware_type, remaining) =
            <u16>::read_from_prefix(frame).map_err(|_| ErrorType::InvalidArgument)?;
        let (protocol_type, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;
        let (hw_address_length, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;
        let (protocol_length, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;
        let (operation, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;

        if hw_address_length != 6 || protocol_length != 4 {
            return Err(ErrorType::InvalidArgument);
        }

        let operation = match u16::from_be(operation) {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            _ => return Err(ErrorType::InvalidArgument),
        };

        let (sender_hw, remaining) =
            <[u8; 6]>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;
        let (sender_ip, remaining) =
            <[u8; 4]>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;
        let (target_hw, remaining) =
            <[u8; 6]>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;
        let (target_ip, _) =
            <[u8; 4]>::read_from_prefix(remaining).map_err(|_| ErrorType::InvalidArgument)?;

        Ok(Self {
            hardware_type: u16::from_be(hardware_type),
            protocol_type: u16::from_be(protocol_type),
            hw_address_length,
            protocol_length,
            operation,
            sender_hw: sender_hw.into(),
            sender_ip,
            target_hw: target_hw.into(),
            target_ip,
        })
    }
}

impl EthFrame<'_> for Arp {
    const TYPE: FrameType = FrameType::ARP;

    fn serialize(&self, output: &mut [u8]) -> Result<usize, ErrorType> {
        if output.len() < ARP_PACKET_SIZE {
            return Err(ErrorType::BufferTooSmall);
        }

        output[0..2].copy_from_slice(&self.hardware_type.to_be_bytes());
        output[2..4].copy_from_slice(&self.protocol_type.to_be_bytes());
        output[4] = self.hw_address_length;
        output[5] = self.protocol_length;
        output[6..8].copy_from_slice(&(self.operation as u16).to_be_bytes());

        let sender_hw = u64::from(self.sender_hw).to_ne_bytes();
        output[8..14].copy_from_slice(&sender_hw[..6]);
        output[14..18].copy_from_slice(&self.sender_ip);

        let target_hw = u64::from(self.target_hw).to_ne_bytes();
        output[18..24].copy_from_slice(&target_hw[..6]);
        output[24..28].copy_from_slice(&self.target_ip);

        Ok(ARP_PACKET_SIZE)
    }

    fn serialize_len(&self) -> usize {
        ARP_PACKET_SIZE
    }
}
