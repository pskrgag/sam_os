use bitmask::bitmask;
use rtl::error::ErrorType;

// Error interrupt bits
bitmask! {
    pub mask SdhciErrors: u16 where flags SdhciError {
        Timeout = 1,
        BlockGap = 4,
        CommandIndex = 8,
    }
}

impl From<SdhciError> for ErrorType {
    fn from(err: SdhciError) -> ErrorType {
        match err {
            SdhciError::Timeout => ErrorType::TryAgain,
            _ => ErrorType::InternalError,
        }
    }
}
