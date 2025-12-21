use super::error::ErrorType;
use bitmask::bitmask;
use super::handle::Handle;

bitmask! {
    pub mask Signals: u8 where flags Signal {
        None = 0,
        MessageReady = (1 << 0),
    }
}

impl TryFrom<usize> for Signals {
    type Error = ErrorType;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let max_set = 64 - value.leading_zeros();

        if max_set > 2 {
            Err(ErrorType::InvalidArgument)
        } else {
            Ok(Signals { mask: value as u8 })
        }
    }
}

impl Default for Signals {
    fn default() -> Self {
        Signals::from(Signal::None)
    }
}

#[repr(C)]
pub struct WaitEntry {
    pub handle: Handle,
    pub waitfor: Signals,
    pub pendind: Signals,
}
