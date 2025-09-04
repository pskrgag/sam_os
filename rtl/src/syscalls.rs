use bitflags::bitflags;

bitflags! {
    pub struct SyscallList: usize {
        const SYS_WRITE = 0;
        const SYS_INVOKE = 1;
        const SYS_YIELD = 2;
        const SYS_CREATE_TASK = 3;
        const SYS_CREATE_PORT = 4;
        const SYS_VM_ALLOCATE = 5;
        const SYS_CREATE_VMO  = 6;
        const SYS_VM_FREE = 7;
        const SYS_MAP_VMO = 8;
        const SYS_MAP_PHYS = 9;
        const SYS_TASK_START = 10;
        const SYS_TASK_GET_VMS = 11;
        const SYS_CLOSE_HANDLE = 12;
        const SYS_PORT_CALL = 13;
        const SYS_PORT_SEND_WAIT = 14;
        const SYS_PORT_RECEIVE = 15;
        const SYS_CLONE_HANDLE = 16;
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
