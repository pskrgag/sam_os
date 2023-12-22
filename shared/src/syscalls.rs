use bitflags::bitflags;

bitflags! {
    pub struct SyscallList: usize {
        const SYS_WRITE = 0;
        const SYS_VM_ALLOCATE = 1;
    }
}

impl From<usize> for SyscallList {
    fn from(value: usize) -> Self {
        Self::from_bits(value).unwrap()
    }
}

impl From<SyscallList> for usize {
    fn from(value: SyscallList) -> Self {
        value.bits()
    }
}
