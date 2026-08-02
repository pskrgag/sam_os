use super::frame::{EthFrame, EthFrameType, FrameType};
use crate::crc::checksum;
use crate::ip::v4::IPv4 as IPv4Address;
use rtl::error::ErrorType;
use zerocopy::FromBytes;

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
            _ => todo!("{value}"),
        }
    }
}

/// IPv4 frame
#[derive(Debug)]
pub struct IPv4<'a> {
    ds_fields: u8,
    length: u16,
    id: u16,
    flags_fragment_offset: u16,
    ttl: u8,
    protocol: Protocol,
    src: IPv4Address,
    dst: IPv4Address,
    data: &'a [u8],
}

impl<'packet> IPv4<'packet> {
    const HEADER_LEN: usize = 20;

    pub fn destination(&self) -> IPv4Address {
        self.dst
    }

    pub fn source(&self) -> IPv4Address {
        self.src
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    pub fn payload<T>(&self) -> Result<T, ErrorType>
    where
        T: Ipv4PacketType + TryFrom<&'packet [u8], Error = ErrorType>,
    {
        if self.protocol() == T::TYPE {
            T::try_from(self.data)
        } else {
            Err(ErrorType::NotFound)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn write_header(
        header: &mut [u8],
        ds_fields: u8,
        total_len: u16,
        id: u16,
        flags_fragment_offset: u16,
        ttl: u8,
        protocol: Protocol,
        src: IPv4Address,
        dst: IPv4Address,
    ) {
        header[0] = 0x45;
        header[1] = ds_fields;
        header[2..4].copy_from_slice(&total_len.to_be_bytes());
        header[4..6].copy_from_slice(&id.to_be_bytes());
        header[6..8].copy_from_slice(&flags_fragment_offset.to_be_bytes());
        header[8] = ttl;
        header[9] = protocol as u8;
        header[10..12].fill(0);
        header[12..16].copy_from_slice(src.as_slice());
        header[16..20].copy_from_slice(dst.as_slice());

        let checksum = checksum(header);
        header[10..12].copy_from_slice(&checksum.to_be_bytes());
    }
}

#[derive(Debug)]
pub struct IPv4Out<T> {
    dst: IPv4Address,
    src: IPv4Address,
    ttl: u8,
    payload: T,
}

impl<T> IPv4Out<T> {
    pub fn new(dst: IPv4Address, src: IPv4Address, payload: T) -> Self {
        Self {
            dst,
            src,
            ttl: 64,
            payload,
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for IPv4<'a> {
    type Error = ErrorType;

    fn try_from(frame: &'a [u8]) -> Result<Self, Self::Error> {
        let (version_ihl, remaining) =
            <u8>::read_from_prefix(frame).map_err(|_| ErrorType::BufferTooSmall)?;

        if version_ihl >> 4 != 4 {
            return Err(ErrorType::InvalidArgument);
        }

        let ihl = version_ihl & 0x0f;
        if ihl < 5 {
            return Err(ErrorType::InvalidArgument);
        }

        let header_length = ihl as usize * 4;
        if frame.len() < header_length {
            return Err(ErrorType::BufferTooSmall);
        }

        let (ds_fields, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (length, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let length = u16::from_be(length);

        if length as usize > frame.len() {
            return Err(ErrorType::BufferTooSmall);
        }
        if (length as usize) < header_length {
            return Err(ErrorType::InvalidArgument);
        }

        let (id, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (flags_fragment_offset, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (ttl, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (protocol, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (_crc, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;

        // header::crc + crc sums to 0xffff. With inversion it becomes 0
        if checksum(&frame[0..header_length]) != 0 {
            return Err(ErrorType::InvalidArgument);
        }

        let (src, remaining) =
            <[u8; 4]>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
        let (dst, _) =
            <[u8; 4]>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;

        Ok(Self {
            ds_fields,
            length,
            id: u16::from_be(id),
            flags_fragment_offset: u16::from_be(flags_fragment_offset),
            ttl,
            protocol: protocol.try_into()?,
            src: src.into(),
            dst: dst.into(),
            data: &frame[header_length..length as usize],
        })
    }
}

impl EthFrameType for IPv4<'_> {
    const TYPE: FrameType = FrameType::IPv4;
}

impl<T: Ipv4Packet> EthFrameType for IPv4Out<T> {
    const TYPE: FrameType = FrameType::IPv4;
}

impl<T: Ipv4Packet> EthFrame for IPv4Out<T> {
    fn serialize(&self, output: &mut [u8]) -> Result<usize, ErrorType> {
        let length = self.serialize_len();
        let total_len = u16::try_from(length).map_err(|_| ErrorType::InvalidArgument)?;
        if output.len() < length {
            return Err(ErrorType::BufferTooSmall);
        }

        let payload_len = self
            .payload
            .serialize(&mut output[IPv4::HEADER_LEN..length])?;
        if payload_len != length - IPv4::HEADER_LEN {
            return Err(ErrorType::InvalidArgument);
        }

        IPv4::write_header(
            &mut output[..IPv4::HEADER_LEN],
            0,
            total_len,
            0,
            0,
            self.ttl,
            T::TYPE,
            self.src,
            self.dst,
        );
        Ok(length)
    }

    fn serialize_len(&self) -> usize {
        IPv4::HEADER_LEN + self.payload.serialize_len()
    }
}

pub trait Ipv4PacketType {
    const TYPE: Protocol;
}

pub trait Ipv4Packet: Ipv4PacketType {
    fn serialize(&self, output: &mut [u8]) -> Result<usize, ErrorType>;
    fn serialize_len(&self) -> usize;
}
