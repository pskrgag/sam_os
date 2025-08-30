use bitflags::bitflags;
use bytemuck::*;

bitflags! {
    #[derive(Zeroable, Pod)]
    #[repr(C)]
    pub struct ErrorType: usize {
        const OK = 0;
        const INVALID_ARGUMENT = 1;
        const NO_OPERATION = 2;
        const FAULT = 3;
        const NO_MEMORY = 4;
        const INVALID_HANDLE = 5;
        const TASK_DEAD = 6;
        const TRY_AGAIN = 7;
        const ALREADY_EXIST = 8;
        const NOT_FOUND = 9;
    }
}

impl From<ErrorType> for usize {
    fn from(value: ErrorType) -> Self {
        value.bits()
    }
}

impl From<usize> for ErrorType {
    fn from(value: usize) -> Self {
        ErrorType::from_bits(value).unwrap()
    }
}
