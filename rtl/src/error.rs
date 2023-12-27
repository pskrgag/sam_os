use bitflags::bitflags;

bitflags! {
    pub struct ErrorType: usize {
        const OK = 0;
        const INVALID_ARGUMENT = 1;
        const NO_OPERATION = 2;
        const FAULT = 3;
        const NO_MEMORY = 4;
    }
}

impl From<ErrorType> for usize {
    fn from(value: ErrorType) -> Self {
        value.bits()
    }
}
