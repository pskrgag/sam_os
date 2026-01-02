use bitvec::prelude::*;
use core::ops::Range;

#[derive(Default)]
pub struct Response(BitArray<[u32; 4]>);

impl Into<Response> for [u32; 4] {
    fn into(self: [u32; 4]) -> Response {
        Response(self.into())
    }
}

impl Response {
    pub fn range(&self, r: Range<usize>) -> &BitSlice<u32> {
        &self.0[r]
    }
}
