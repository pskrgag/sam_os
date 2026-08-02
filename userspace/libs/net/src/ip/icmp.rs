use crate::{
    crc::checksum,
    eth::ipv4::{Ipv4Packet, Ipv4PacketType, Protocol},
};
use core::mem::size_of;
use rtl::error::ErrorType;
use zerocopy::FromBytes;

#[derive(Debug)]
#[repr(u8)]
pub enum TypeRaw {
    EchoRequest = 8,
    EchoReply = 0,
}

#[derive(Debug)]
pub enum Icmp<'a> {
    EchoRequest {
        id: u16,
        seq: u16,
        payload: &'a [u8],
    },
    EchoReply {
        id: u16,
        seq: u16,
        payload: &'a [u8],
    },
}

impl<'a> TryFrom<&'a [u8]> for Icmp<'a> {
    type Error = ErrorType;

    fn try_from(frame: &'a [u8]) -> Result<Self, Self::Error> {
        let (tp, remaining) =
            <u8>::read_from_prefix(frame).map_err(|_| ErrorType::BufferTooSmall)?;

        let (code, remaining) =
            <u8>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;

        let (_crc, remaining) =
            <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;

        if checksum(frame) != 0 {
            return Err(ErrorType::InvalidArgument);
        }

        match tp {
            _ if tp == TypeRaw::EchoRequest as u8 || tp == TypeRaw::EchoReply as u8 => {
                if code != 0 {
                    return Err(ErrorType::InvalidArgument);
                }

                let (id, remaining) =
                    <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;

                let (seq, payload) =
                    <u16>::read_from_prefix(remaining).map_err(|_| ErrorType::BufferTooSmall)?;
                let id = u16::from_be(id);
                let seq = u16::from_be(seq);

                if tp == TypeRaw::EchoRequest as u8 {
                    Ok(Self::EchoRequest { id, seq, payload })
                } else {
                    Ok(Self::EchoReply { id, seq, payload })
                }
            }
            _ => Err(ErrorType::InvalidArgument),
        }
    }
}

impl Ipv4PacketType for Icmp<'_> {
    const TYPE: Protocol = Protocol::ICMP;
}

impl Ipv4Packet for Icmp<'_> {
    fn serialize(&self, output: &mut [u8]) -> Result<usize, ErrorType> {
        let length = self.serialize_len();
        if output.len() < length {
            return Err(ErrorType::BufferTooSmall);
        }

        let (packet_type, id, seq, payload) = match self {
            Self::EchoRequest { id, seq, payload } => (TypeRaw::EchoRequest, *id, *seq, *payload),
            Self::EchoReply { id, seq, payload } => (TypeRaw::EchoReply, *id, *seq, *payload),
        };

        output[0] = packet_type as u8;
        output[1] = 0;
        output[2..4].fill(0);
        output[4..6].copy_from_slice(&id.to_be_bytes());
        output[6..8].copy_from_slice(&seq.to_be_bytes());
        output[8..length].copy_from_slice(payload);

        let checksum = checksum(&output[..length]);
        output[2..4].copy_from_slice(&checksum.to_be_bytes());
        Ok(length)
    }

    fn serialize_len(&self) -> usize {
        match self {
            Self::EchoReply { payload, .. } => size_of::<u16>() * 2 + payload.len() + 4,
            Self::EchoRequest { payload, .. } => size_of::<u16>() * 2 + payload.len() + 4,
        }
    }
}
