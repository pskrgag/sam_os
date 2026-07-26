use core::fmt::{self, Debug, Formatter};
use rtl::error::ErrorType;

/// Mac address
pub struct Mac([u8; 6]);

impl Mac {
    pub fn from_raw(raw: u64) -> Result<Self, ErrorType> {
        let mask = ((1 << 16) - 1) << 48;
        let mut res = [0u8; 6];

        if raw & mask != 0 {
            return Err(ErrorType::InvalidArgument);
        }

        res.copy_from_slice(&raw.to_le_bytes()[0..6]);
        Ok(Self(res))
    }
}

impl Debug for Mac {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}:", self.0[0])?;
        write!(f, "{:02x}:", self.0[1])?;
        write!(f, "{:02x}:", self.0[2])?;
        write!(f, "{:02x}:", self.0[3])?;
        write!(f, "{:02x}:", self.0[4])?;
        write!(f, "{:02x}", self.0[5])
    }
}
