use super::mac::Mac;
use alloc::vec;
use alloc::vec::Vec;
use rtl::error::ErrorType;
use zerocopy::FromBytes;

pub trait EthFrameType {
    const TYPE: FrameType;
}

pub trait EthFrame: EthFrameType {
    fn serialize(&self, output: &mut [u8]) -> Result<usize, ErrorType>;
    fn serialize_len(&self) -> usize;
}

#[derive(Debug)]
#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum FrameType {
    ARP = 0x0806,
    IPv4 = 0x0800,
    IPv6 = 0x86DD,
    // QTagged = 0x8100,
}

impl TryFrom<u16> for FrameType {
    type Error = ErrorType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::ARP as u16 => Ok(Self::ARP),
            _ if value == Self::IPv6 as u16 => Ok(Self::IPv6),
            _ if value == Self::IPv4 as u16 => Ok(Self::IPv4),
            // _ if value == Self::QTagged as u16 => Ok(Self::QTagged),
            _ => Err(ErrorType::InvalidArgument),
        }
    }
}

// Ethernet frame
#[derive(Debug)]
pub struct Frame<'a> {
    dst: Mac,
    src: Mac,
    tp: FrameType,
    payload: &'a [u8],
}

impl Frame<'_> {
    pub fn serialize<T: EthFrame>(dst: Mac, src: Mac, payload: T) -> Vec<u8> {
        let mut data = vec![0; 6 + 6 + 2 + payload.serialize_len()];

        let dst = u64::from(dst).to_ne_bytes();
        data[0..6].copy_from_slice(&dst[..6]);

        let src = u64::from(src).to_ne_bytes();
        data[6..12].copy_from_slice(&src[..6]);

        data[12..14].copy_from_slice(&(T::TYPE as u16).to_be_bytes());

        let payload_len = payload.serialize(&mut data[14..]).unwrap();
        assert_eq!(payload_len, data.len() - 14);

        data
    }

    pub fn destination(&self) -> Mac {
        self.dst
    }

    pub fn source(&self) -> Mac {
        self.src
    }

    pub fn frame_type(&self) -> FrameType {
        self.tp
    }

    pub fn raw_payload(&self) -> &[u8] {
        self.payload
    }

    pub fn payload<'a, T>(&'a self) -> Result<T, ErrorType>
    where
        T: EthFrameType + TryFrom<&'a [u8], Error = ErrorType>,
    {
        if self.tp == T::TYPE {
            T::try_from(self.raw_payload())
        } else {
            Err(ErrorType::NotFound)
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Frame<'a> {
    type Error = ErrorType;

    fn try_from(frame: &'a [u8]) -> Result<Self, Self::Error> {
        let (dst, remaining): (Mac, _) = <[u8; 6]>::read_from_prefix(frame)
            .map(|x| (x.0.into(), x.1))
            .map_err(|_| ErrorType::BufferTooSmall)?;

        let (src, remaining): (Mac, _) = <[u8; 6]>::read_from_prefix(remaining)
            .map(|x| (x.0.into(), x.1))
            .map_err(|_| ErrorType::BufferTooSmall)?;

        let (tp, remaining) = <u16>::read_from_prefix(remaining)
            .map_err(|_| ErrorType::BufferTooSmall)
            .and_then(|(raw, remaining)| {
                FrameType::try_from(u16::from_be(raw)).map(|tp| (tp, remaining))
            })?;

        Ok(Self {
            dst,
            src,
            tp,
            payload: remaining,
        })
    }
}
