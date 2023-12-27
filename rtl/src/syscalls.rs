use bitflags::bitflags;

bitflags! {
    pub struct SyscallList: usize {
        const SYS_WRITE = 0;

        /// VMM
        const SYS_VM_ALLOCATE = 1;
        const SYS_VM_CREATE_VM_OBJECT = 2;

        /// Task
        const SYS_TASK_CREATE_FROM_VMO = 3;
        const SYS_TASK_START = 4;
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
