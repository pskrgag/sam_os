#[derive(Default)]
pub struct Response([u32; 4]);

impl Into<Response> for [u32; 4] {
    fn into(self: [u32; 4]) -> Response {
        Response(self)
    }
}

impl Response {
    pub fn check_bit(&self, bit: u8) -> bool {
        let index = bit / 32;
        let offset = bit % 32;

        self.0[index as usize] & (1 << offset) != 0
    }
}
