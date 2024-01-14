use bitflags::bitflags;

bitflags! {
    pub struct ErrorType: usize {
        const OK = 0;
        const INVALID_ARGUMENT = 1;
        const NO_OPERATION = 2;
        const FAULT = 4;
        const NO_MEMORY = 8;
        const INVALID_HANDLE = 16;
        const TASK_DEAD = 32;
        const TRY_AGAIN = 64;
    }
}

impl From<ErrorType> for usize {
    fn from(value: ErrorType) -> Self {
        value.bits()
    }
}
