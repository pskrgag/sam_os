use core::mem::size_of;
use rtl::error::ErrorType;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

pub trait Header: FromBytes + Immutable + KnownLayout + Unaligned + Sized + IntoBytes {
    type Error: From<ErrorType>;

    fn header_len(data: &[u8]) -> Result<usize, Self::Error>;

    fn parse(data: &[u8]) -> Result<&Self, Self::Error> {
        Self::header_len(data)?;

        Self::ref_from_prefix(data)
            .map(|(header, _)| header)
            .map_err(|_| ErrorType::BufferTooSmall.into())
    }

    fn parse_mut(data: &mut [u8]) -> Result<&mut Self, Self::Error> {
        Self::header_len(data)?;

        Self::mut_from_prefix(data)
            .map(|(header, _)| header)
            .map_err(|_| ErrorType::BufferTooSmall.into())
    }

    fn fixed_len() -> usize {
        size_of::<Self>()
    }
}
