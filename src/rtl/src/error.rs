#[repr(usize)]
#[derive(Debug)]
pub enum ErrorType {
    Ok = 0,
    InvalidArgument = 1,
    NoOperation = 2,
    Fault = 3,
    NoMemory = 4,
    InvalidHandle = 5,
    TaskDead = 6,
    TryAgain = 7,
    AlreadyExists = 8,
    NotFound = 9,
}

impl From<ErrorType> for usize {
    fn from(value: ErrorType) -> Self {
        value as usize
    }
}
