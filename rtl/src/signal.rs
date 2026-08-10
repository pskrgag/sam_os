use super::error::ErrorType;
use super::handle::Handle;
use bitmask::bitmask;

bitmask! {
    pub mask Signals: u8 where flags Signal {
        None = 0,
        MessageReady = (1 << 0),
        TimerReady = (1 << 1),
    }
}

impl TryFrom<usize> for Signals {
    type Error = ErrorType;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let max_set = core::mem::size_of::<usize>() * 8 - value.leading_zeros() as usize;

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
    pub context: usize,
    pub context1: usize,
}
