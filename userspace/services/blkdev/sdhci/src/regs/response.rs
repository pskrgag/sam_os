use bitvec::prelude::*;
use core::ops::Range;

#[derive(Default)]
pub struct ResponseU128(BitArray<[u32; 4]>);

#[derive(Default)]
pub struct ResponseU32(BitArray<u32>);

#[derive(Default)]
pub struct NoResponse;

impl From<[u32; 4]> for ResponseU128 {
    fn from(val: [u32; 4]) -> ResponseU128 {
        let mut new = val.clone();

        for i in 0..4 {
            new[i] <<= 8;

            if i != 0 {
                new[i] |= val[i - 1] >> 24;
            }
        }

        ResponseU128(new.into())
    }
}

impl From<[u32; 4]> for ResponseU32 {
    fn from(val: [u32; 4]) -> ResponseU32 {
        ResponseU32(val[0].into())
    }
}

// TODO: this is not good
impl From<[u32; 4]> for NoResponse {
    fn from(_val: [u32; 4]) -> NoResponse {
        Self
    }
}

impl From<ResponseU128> for [u32; 4] {
    fn from(val: ResponseU128) -> [u32; 4] {
        val.0.into_inner()
    }
}

impl ResponseU128 {
    pub fn range(&self, r: Range<usize>) -> &BitSlice<u32> {
        &self.0[r]
    }
}

impl ResponseU32 {
    pub fn range(&self, r: Range<usize>) -> &BitSlice<u32> {
        &self.0[r]
    }
}

pub trait Response: From<[u32; 4]> {}
impl Response for ResponseU128 {}
impl Response for ResponseU32 {}
impl Response for NoResponse {}
