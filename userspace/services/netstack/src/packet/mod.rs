use alloc::vec::Vec;
use net::header::Header;
use rtl::error::ErrorType;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// An owned network packet with a cursor and recorded protocol header offsets.
#[derive(Debug)]
pub struct Packet {
    data: Vec<u8>,
    start: usize,
    end: usize,
    mac_header: Option<usize>,
    network_header: Option<usize>,
    transport_header: Option<usize>,
}

impl Packet {
    pub fn new(data: Vec<u8>) -> Self {
        let end = data.len();

        Self {
            data,
            start: 0,
            end,
            mac_header: None,
            network_header: None,
            transport_header: None,
        }
    }

    pub fn with_capacity(headroom: usize, payload: usize) -> Self {
        Self {
            data: vec![0; headroom + payload],
            start: headroom,
            end: headroom,
            mac_header: None,
            network_header: None,
            transport_header: None,
        }
    }

    fn head_room(&self) -> usize {
        self.start
    }

    pub fn push_header<T>(&mut self, header: &T)
    where
        T: Header<Error = ErrorType>,
    {
        let len = header.as_bytes().len();
        let old_head = self.head_room();

        let (new_head, overflow) = self.head_room().overflowing_sub(len);

        if overflow {
            panic!("todo");
        }

        self.data[new_head..old_head].copy_from_slice(header.as_bytes());
        self.start = new_head;
    }

    pub fn push_payload(&mut self, payload: &[u8]) {
        let new_end = self
            .end
            .checked_add(payload.len())
            .expect("packet payload length overflow");

        if new_end > self.data.len() {
            panic!("packet tailroom exhausted");
        }

        self.data[self.end..new_end].copy_from_slice(payload);
        self.end = new_end;
    }

    pub fn payload_len(&self) -> usize {
        self.end - self.start
    }

    pub fn parse_mac_header<T>(&mut self) -> Result<&T, ErrorType>
    where
        T: Header<Error = ErrorType>,
    {
        if self.mac_header.is_some() {
            return Err(ErrorType::AlreadyExists);
        }

        let (offset, header_len) = self.parse_header::<T>()?;

        self.mac_header = Some(offset);
        self.start += header_len;
        self.header_at(offset)
    }

    pub fn parse_network_header_mut<T>(&mut self) -> Result<&mut T, ErrorType>
    where
        T: Header<Error = ErrorType>,
    {
        if self.network_header.is_some() {
            return Err(ErrorType::AlreadyExists);
        }

        let (offset, header_len) = self.parse_header::<T>()?;

        self.network_header = Some(offset);
        self.start += header_len;
        Ok(self.header_mut(offset))
    }

    pub fn parse_transport_header<T>(&mut self) -> Result<&T, ErrorType>
    where
        T: Header<Error = ErrorType>,
    {
        if self.transport_header.is_some() {
            return Err(ErrorType::AlreadyExists);
        }

        let (offset, header_len) = self.parse_header::<T>()?;

        self.transport_header = Some(offset);
        self.start += header_len;
        self.header_at(offset)
    }

    pub fn mac_header<T>(&self) -> &T
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        self.header(self.mac_header.expect("MAC header was not parsed"))
    }

    pub fn mac_header_mut<T>(&mut self) -> Option<&mut T>
    where
        T: FromBytes + IntoBytes + KnownLayout + Immutable,
    {
        self.mac_header.map(|x| self.header_mut(x))
    }

    pub fn network_header<T>(&self) -> &T
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        self.header(self.network_header.expect("network header was not parsed"))
    }

    pub fn network_header_mut<T>(&mut self) -> &mut T
    where
        T: FromBytes + IntoBytes + KnownLayout + Immutable,
    {
        self.header_mut(self.network_header.expect("network header was not parsed"))
    }

    pub fn transport_header<T>(&self) -> &T
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        self.header(
            self.transport_header
                .expect("transport header was not parsed"),
        )
    }

    pub fn transport_header_mut<T>(&mut self) -> &mut T
    where
        T: FromBytes + IntoBytes + KnownLayout + Immutable,
    {
        self.header_mut(
            self.transport_header
                .expect("transport header was not parsed"),
        )
    }

    pub fn trim_network(&mut self, len: usize) -> Result<(), ErrorType> {
        let network = self.network_header.ok_or(ErrorType::NotFound)?;
        let end = network.checked_add(len).ok_or(ErrorType::BufferTooBig)?;

        if end > self.end || end < self.start {
            return Err(ErrorType::BufferTooSmall);
        }

        self.end = end;
        Ok(())
    }

    pub fn payload(&self) -> &[u8] {
        &self.data[self.start..self.end]
    }

    pub fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.data[self.start..self.end]
    }

    pub fn into_frame(mut self) -> Vec<u8> {
        let start = self.mac_header.unwrap_or(self.start);

        self.data.truncate(self.end);
        self.data.drain(..start);
        self.data
    }

    fn parse_header<T>(&self) -> Result<(usize, usize), ErrorType>
    where
        T: Header<Error = ErrorType>,
    {
        let data = self
            .data
            .get(self.start..self.end)
            .ok_or(ErrorType::Fault)?;
        let header_len = T::header_len(data)?;
        let next = self
            .start
            .checked_add(header_len)
            .ok_or(ErrorType::BufferTooBig)?;

        if next > self.end {
            return Err(ErrorType::BufferTooSmall);
        }

        Ok((self.start, header_len))
    }

    fn header<T>(&self, offset: usize) -> &T
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        T::ref_from_prefix(&self.data[offset..self.end])
            .expect("recorded header does not fit in packet")
            .0
    }

    fn header_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: FromBytes + IntoBytes + KnownLayout + Immutable,
    {
        T::mut_from_prefix(&mut self.data[offset..self.end])
            .expect("recorded header does not fit in packet")
            .0
    }

    fn header_at<T>(&self, offset: usize) -> Result<&T, ErrorType>
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        T::ref_from_prefix(&self.data[offset..self.end])
            .map(|(header, _)| header)
            .map_err(|_| ErrorType::BufferTooSmall)
    }
}
