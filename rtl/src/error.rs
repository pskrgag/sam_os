#[repr(usize)]
#[derive(Debug)]
pub enum ErrorType {
    InvalidArgument = 1,
    NoOperation = 2,
    Fault = 3,
    NoMemory = 4,
    InvalidHandle = 5,
    TaskDead = 6,
    TryAgain = 7,
    AlreadyExists = 8,
    NotFound = 9,
    Generic = 10,
    InternalError = 11,
    BufferTooSmall = 12,
    BufferTooBig = 13,
    WouldBlock = 14,
}

impl From<ErrorType> for &str {
    fn from(err: ErrorType) -> &'static str {
        match err {
            ErrorType::WouldBlock => "will block",
            ErrorType::NoMemory => "out of memory",
            ErrorType::Fault => "invalid address",
            ErrorType::AlreadyExists => "already exists",
            ErrorType::NotFound => "not found",
            _ => todo!(),
        }
    }
}

impl From<ErrorType> for usize {
    fn from(value: ErrorType) -> Self {
        value as usize
    }
}
